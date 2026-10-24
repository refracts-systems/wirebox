"""Negative controls for the independent Linux corpus, using recorded captures."""
import copy
import hashlib
import json
from pathlib import Path
import shutil
import tempfile
import unittest
from jsonschema.exceptions import ValidationError

from compare import checksum, compare_case
from linux_lab import slug
from validate import read, safe_path, validate_record


ROOT = Path(__file__).resolve().parents[2]


def write(path, value):
    path.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")


class CorpusTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        records = sorted((ROOT / "corpus/devices").rglob("record.json"))
        cls.good = next(p for p in reversed(records) if read(p)["status"] == "pass")
        cls.failed = next(p for p in records if read(p)["status"] == "fail")
        cls.contract = read(cls.good.parent / "scenario.json")
        cls.simulator = read(cls.good.parent / "S01.json")
        cls.rows, cls.stats = [], []
        for role in ("source", "router"):
            case = cls.good.parent / "S01-r1"
            cls.rows.extend(json.loads(line) for line in (case / f"{role}.jsonl").read_text().splitlines())
            cls.stats.append(read(case / f"{role}.stats.json"))

    def copy_record(self):
        temporary = tempfile.TemporaryDirectory(prefix="wyrlex-corpus-test-")
        self.addCleanup(temporary.cleanup)
        folder = Path(temporary.name) / self.good.parent.name
        shutil.copytree(self.good.parent, folder)
        return folder / "record.json"

    def update_artifact(self, record_path, relative):
        record = read(record_path)
        artifact = next(a for a in record["artifacts"] if a["path"] == relative)
        data = (record_path.parent / relative).read_bytes()
        artifact.update(sha256=hashlib.sha256(data).hexdigest(), bytes=len(data))
        write(record_path, record)

    def test_successful_record_and_known_failed_record_remain_distinct(self):
        self.assertEqual(validate_record(ROOT, self.good, True)["comparison"], "pass")
        self.assertEqual(validate_record(ROOT, self.failed)["comparison"], "fail")
        with self.assertRaisesRegex(ValueError, "known mismatches"):
            validate_record(ROOT, self.failed, True)

    def test_corrupt_capture_rejected_by_hash(self):
        record = self.copy_record()
        path = record.parent / "S01-r1/source.pcap"
        data = bytearray(path.read_bytes())
        data[-1] ^= 1
        path.write_bytes(data)
        with self.assertRaisesRegex(ValueError, "hash mismatch"):
            validate_record(ROOT, record)

    def test_rehashed_pcap_still_must_match_jsonl(self):
        record = self.copy_record()
        relative = "S01-r1/source.pcap"
        path = record.parent / relative
        data = bytearray(path.read_bytes())
        data[-1] ^= 1
        path.write_bytes(data)
        self.update_artifact(record, relative)
        with self.assertRaisesRegex(ValueError, "pcap and JSONL bytes differ"):
            validate_record(ROOT, record)

    def test_missing_case_cannot_pass_after_report_is_rehashed(self):
        record = self.copy_record()
        path = record.parent / "comparison.json"
        value = read(path)
        value["comparisons"].pop()
        write(path, value)
        self.update_artifact(record, "comparison.json")
        with self.assertRaisesRegex(ValueError, "missing or duplicate case"):
            validate_record(ROOT, record)

    def test_fabricated_check_actual_is_rejected_after_rehash(self):
        record = self.copy_record()
        path = record.parent / "comparison.json"
        value = read(path)
        value["comparisons"][0]["checks"][0]["actual"] = "fabricated"
        write(path, value)
        self.update_artifact(record, "comparison.json")
        with self.assertRaisesRegex(ValueError, "does not reproduce"):
            validate_record(ROOT, record)

    def test_missing_required_invariant_is_rejected(self):
        record = self.copy_record()
        path = record.parent / "comparison.json"
        value = read(path)
        value["comparisons"][0]["checks"].pop()
        write(path, value)
        self.update_artifact(record, "comparison.json")
        with self.assertRaisesRegex(ValueError, "missing or duplicate invariant"):
            validate_record(ROOT, record)

    def test_unknown_schema_version_is_rejected(self):
        record = self.copy_record()
        value = read(record)
        value["schema_version"] = 99
        write(record, value)
        with self.assertRaises(ValidationError):
            validate_record(ROOT, record)

    def test_comparator_revision_cannot_be_switched_under_existing_scenario(self):
        record = self.copy_record()
        path = record.parent / "comparison.json"
        value = read(path)
        value["comparator_revision"] = 1
        write(path, value)
        self.update_artifact(record, "comparison.json")
        with self.assertRaisesRegex(ValueError, "revisions differ"):
            validate_record(ROOT, record)

    def test_path_traversal_absolute_and_windows_paths_are_rejected(self):
        for path in ("../escape", "/etc/passwd", "C:/secret", "a\\b"):
            with self.subTest(path=path), self.assertRaises(ValueError):
                safe_path(ROOT, path)

    def test_environment_path_components_are_sanitized(self):
        self.assertEqual(slug("6.18.33.2-microsoft-standard-WSL2"), "6.18.33.2-microsoft-standard-wsl2")
        self.assertNotIn("/", slug("../other/model"))
        with self.assertRaises(ValueError):
            slug("../")

    def test_no_capture_cannot_be_a_success(self):
        result = compare_case("S01", self.simulator, [], self.stats, self.contract)
        self.assertEqual(result["status"], "fail")
        self.assertIn("source_receives_expected_icmp", result["mismatches"])

    def test_wrong_ttl_with_correct_checksum_is_rejected(self):
        rows = copy.deepcopy(self.rows)
        changed = 0
        for row in rows:
            packet = bytearray.fromhex(row["hex"])
            if row["role"] == "router" and row["interface"] == "eth1" and row["packet_type"] == 4 and len(packet) >= 42 and packet[12:14] == b"\x08\x00" and packet[34] == 8:
                packet[22] -= 1
                packet[24:26] = b"\0\0"
                packet[24:26] = checksum(bytes(packet[14:34])).to_bytes(2, "big")
                row["hex"] = packet.hex()
                changed += 1
        self.assertGreater(changed, 0)
        result = compare_case("S01", self.simulator, rows, self.stats, self.contract)
        self.assertIn("forwarded_request_or_no_forward", result["mismatches"])
        self.assertNotIn("valid_ip_and_icmp_checksums", result["mismatches"])

    def test_capture_drops_are_not_accepted(self):
        stats = copy.deepcopy(self.stats)
        stats[0]["kernel_drops"] = 1
        self.assertIn("capture_no_kernel_drops", compare_case("S01", self.simulator, self.rows, stats, self.contract)["mismatches"])

    def test_classifier_revision_preserves_initial_instrumentation_failure(self):
        failed = read(self.failed.parent / "comparison.json")
        case = next(r for r in failed["comparisons"] if r["mismatches"])
        folder = self.failed.parent / f"{case['case']}-r{case['repetition']}"
        rows, stats = [], []
        for role in ("source", "router"):
            rows.extend(json.loads(line) for line in (folder / f"{role}.jsonl").read_text().splitlines())
            stats.append(read(folder / f"{role}.stats.json"))
        simulator = read(self.failed.parent / f"{case['case']}.json")
        contract = read(self.failed.parent / "scenario.json")
        self.assertIn("valid_ip_and_icmp_checksums", compare_case(case["case"], simulator, rows, stats, contract, revision=1)["mismatches"])
        self.assertEqual(compare_case(case["case"], simulator, rows, stats, contract, revision=2)["status"], "pass")


if __name__ == "__main__":
    unittest.main()
