import ghidra.app.script.GhidraScript;
import ghidra.app.decompiler.DecompInterface;
import ghidra.app.decompiler.DecompileResults;
import ghidra.program.model.listing.Function;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.io.PrintWriter;

// Read-only extraction. Marker presence is evidence for review, not protocol conformance.
public class WireboxPtAudit extends GhidraScript {
    private String quote(String s) {
        if (s == null) return "null";
        return "\"" + s.replace("\\", "\\\\").replace("\"", "\\\"")
            .replace("\r", "\\r").replace("\n", "\\n").replace("\t", "\\t") + "\"";
    }

    public void run() throws Exception {
        String[] args = getScriptArgs();
        if (args.length != 2) throw new IllegalArgumentException("output-directory expected-sha256");
        String sha = currentProgram.getExecutableSHA256();
        if (!args[1].equalsIgnoreCase(sha)) throw new IllegalArgumentException("Project SHA-256 differs from target");
        Path dir = Path.of(args[0]);
        Files.createDirectories(dir);
        String[] addresses = {"141d1d3e0", "141d22cf0", "14094c530", "142580fc0"};
        String[][] markers = {
            {"Simulation::CScheduler::vftable", "QTimer::stop", "QSemaphore::~QSemaphore", "QWaitCondition::~QWaitCondition", "* 2 + 2", "* 0x10"},
            {"SimulationTimer", "getIpcTimer_implementation_error", "Simulation::CIpcTimer::RTTI_Type_Descriptor"},
            {"routed via RIB", "ICMP: unreachable host", "ICMP: time exceeded (time to live)"},
            {"Buffer Full -- %1", "Clear Event List", "View Previous Events", "QMessageBox::setText"}
        };
        DecompInterface decompiler = new DecompInterface();
        decompiler.toggleCCode(true);
        decompiler.toggleSyntaxTree(false);
        if (!decompiler.openProgram(currentProgram)) throw new IllegalStateException("Cannot open decompiler");
        try (PrintWriter raw = new PrintWriter(Files.newBufferedWriter(dir.resolve("decomp.txt"), StandardCharsets.UTF_8));
             PrintWriter report = new PrintWriter(Files.newBufferedWriter(dir.resolve("ghidra-audit.json"), StandardCharsets.UTF_8))) {
            report.println("{\"method\":\"Ghidra headless targeted static decompilation\",\"target_sha256\":" + quote(sha) + ",\"functions\":[");
            for (int i = 0; i < addresses.length; i++) {
                Function f = getFunctionAt(toAddr(addresses[i]));
                if (f == null) throw new IllegalStateException("Missing function " + addresses[i]);
                DecompileResults result = decompiler.decompileFunction(f, 120, monitor);
                boolean ok = result != null && result.decompileCompleted();
                String code = ok ? result.getDecompiledFunction().getC() : "";
                raw.println("FUNCTION " + addresses[i]);
                raw.println(code);
                raw.flush();
                report.print((i == 0 ? "" : ",") + "{\"address\":" + quote(addresses[i]) + ",\"name\":" + quote(f.getName(true))
                    + ",\"body_addresses\":" + f.getBody().getNumAddresses() + ",\"decompile_completed\":" + ok
                    + ",\"error\":" + quote(result == null ? "null result" : result.getErrorMessage()) + ",\"markers\":[");
                for (int m = 0; m < markers[i].length; m++) {
                    report.print((m == 0 ? "" : ",") + "{\"text\":" + quote(markers[i][m]) + ",\"present\":" + code.contains(markers[i][m]) + "}");
                }
                report.println("]}");
                report.flush();
                if (!ok) throw new IllegalStateException("Decompilation failed at " + addresses[i]);
            }
            report.println("]}");
        } finally {
            decompiler.dispose();
        }
        println("Wyrlex PT audit completed; static markers require interpretation.");
    }
}
