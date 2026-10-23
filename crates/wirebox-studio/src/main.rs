use gpui::{
    App, Application, Bounds, Context, FocusHandle, KeyBinding, SharedString, Window, WindowBounds,
    WindowOptions, actions, div, prelude::*, px, rgb, size,
};
use std::{fs::OpenOptions, io::Write, path::PathBuf};
use wirebox_kernel::{ABI_VERSION, BATCH_LIMIT, PROBE_BYTES, ProbeEvent, Session};

actions!(wirebox, [CloseWindow, ReadProbe, ClearSelection]);

#[derive(Clone)]
struct SmokeLog(Option<PathBuf>);

impl SmokeLog {
    fn write(&self, message: &str) {
        if let Some(path) = &self.0 {
            match OpenOptions::new().append(true).create(true).open(path) {
                Ok(mut file) => {
                    let _ = writeln!(file, "{message}");
                }
                Err(error) => eprintln!("smoke log: {error}"),
            }
        }
    }
}

struct Studio {
    focus: FocusHandle,
    session: Option<Session>,
    event: ProbeEvent,
    selected: String,
    detail: String,
    selected_byte: Option<usize>,
    log: SmokeLog,
}

impl Studio {
    fn new(window: &mut Window, cx: &mut Context<Self>, log: SmokeLog) -> Self {
        let mut session = Session::new().expect("create Zig session");
        session.submit_probe(PROBE_BYTES).expect("submit Zig probe");
        let event = session
            .read_probe(BATCH_LIMIT)
            .expect("read Zig probe")
            .remove(0);
        let focus = cx.focus_handle();
        focus.focus(window);
        log.write("opened:abi=1");
        Self {
            focus,
            session: Some(session),
            event,
            selected: "PC1".into(),
            detail: "Fixture host / 10.0.0.2/24".into(),
            selected_byte: None,
            log,
        }
    }

    fn select(&mut self, name: &str, detail: &str, cx: &mut Context<Self>) {
        self.selected = name.into();
        self.detail = detail.into();
        self.selected_byte = None;
        self.log.write(&format!("selected:{name}"));
        cx.notify();
    }

    fn read_probe(&mut self, _: &ReadProbe, _: &mut Window, cx: &mut Context<Self>) {
        let result = self
            .session
            .as_mut()
            .unwrap()
            .submit_probe(PROBE_BYTES)
            .and_then(|()| self.session.as_mut().unwrap().read_probe(BATCH_LIMIT));
        match result {
            Ok(mut events) if events.len() == 1 => {
                self.event = events.remove(0);
                self.log.write(&format!("read_probe:id={}", self.event.id));
            }
            Ok(_) => {
                self.detail = "Unexpected batch count".into();
                self.log.write("probe_error:count");
            }
            Err(error) => {
                self.detail = error.to_string();
                self.log.write("probe_error:kernel");
            }
        }
        cx.notify();
    }
}

impl Drop for Studio {
    fn drop(&mut self) {
        if let Some(session) = self.session.take() {
            match session.close() {
                Ok(()) => self.log.write("session_destroyed"),
                Err(error) => {
                    self.log.write("session_destroy_failed");
                    eprintln!("{error}");
                }
            }
        }
    }
}

impl Render for Studio {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let nodes = [
            ("PC1", "Host", "Fixture host / 10.0.0.2/24"),
            ("SW1", "Switch", "Fixture switch / Ethernet ports 0 and 1"),
            ("R1", "Router", "Fixture router / 10.0.0.1 and 10.0.1.1"),
            ("PC2", "Host", "Fixture host / 10.0.1.2/24"),
        ];
        let mut topology = div().flex().items_center().gap_2().w_full();
        for (index, (name, role, detail)) in nodes.into_iter().enumerate() {
            if index > 0 {
                let link_name = ["PC1–SW1", "SW1–R1", "R1–PC2"][index - 1];
                topology = topology.child(
                    div()
                        .id(("link", index))
                        .w(px(76.))
                        .py_3()
                        .cursor_pointer()
                        .border_b_2()
                        .border_color(rgb(if self.selected == link_name {
                            0x56c4a4
                        } else {
                            0x465363
                        }))
                        .text_xs()
                        .text_color(rgb(0x9ca9b8))
                        .child("100 Mb/s")
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.select(
                                link_name,
                                "Fixture link / delay model not implemented yet",
                                cx,
                            );
                        })),
                );
            }
            topology = topology.child(
                div()
                    .id(("node", index))
                    .flex_1()
                    .p_4()
                    .rounded_lg()
                    .border_1()
                    .border_color(rgb(if self.selected == name {
                        0x56c4a4
                    } else {
                        0x344252
                    }))
                    .bg(rgb(if self.selected == name {
                        0x19392f
                    } else {
                        0x1a2430
                    }))
                    .cursor_pointer()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(div().text_lg().child(name))
                    .child(div().text_xs().text_color(rgb(0x9ca9b8)).child(role))
                    .on_click(cx.listener(move |this, _, _, cx| this.select(name, detail, cx))),
            );
        }

        let mut hex = div().flex().flex_col().gap_2();
        for (row, chunk) in self.event.bytes.chunks(8).enumerate() {
            let mut line = div().flex().gap_2().items_center().child(
                div()
                    .w(px(45.))
                    .text_color(rgb(0x8291a3))
                    .child(format!("{:04x}", row * 8)),
            );
            for (column, byte) in chunk.iter().copied().enumerate() {
                let offset = row * 8 + column;
                line = line.child(
                    div()
                        .id(("byte", offset))
                        .w(px(33.))
                        .h(px(32.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded_sm()
                        .bg(rgb(if self.selected_byte == Some(offset) {
                            0x256653
                        } else {
                            0x202e3c
                        }))
                        .cursor_pointer()
                        .child(format!("{byte:02x}"))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.selected_byte = Some(offset);
                            this.selected = format!("Byte {offset}");
                            this.detail =
                                format!("Offset {offset} / 0x{byte:02x} / decimal {byte}");
                            this.log.write(&format!("byte:{offset}={byte:02x}"));
                            cx.notify();
                        })),
                );
            }
            hex = hex.child(line);
        }

        div().id("studio").track_focus(&self.focus)
            .on_action(|_: &CloseWindow, window, _| window.remove_window())
            .on_action(cx.listener(Self::read_probe))
            .on_action(cx.listener(|this, _: &ClearSelection, _, cx| {
                this.select("PC1", "Fixture host / 10.0.0.2/24", cx);
            }))
            .size_full().p_6().flex().flex_col().gap_5()
            .font_family("Segoe UI").bg(rgb(0x101820)).text_color(rgb(0xe6edf3))
            .child(div().flex().justify_between().items_center()
                .child(div().flex().flex_col().gap_1()
                    .child(div().text_2xl().child("Wirebox"))
                    .child(div().text_sm().text_color(rgb(0x9ca9b8)).child("Infrastructure studio / foundation preview")))
                .child(div().text_sm().px_3().py_2().rounded_md().bg(rgb(0x19392f)).text_color(rgb(0x7fd6b9))
                    .child(format!("PHASE 1   ·   ABI {ABI_VERSION}"))))
            .child(div().p_3().rounded_md().bg(rgb(0x2c2a20)).text_sm().text_color(rgb(0xe5c780))
                .child("Static topology fixture + real Zig/Rust byte transport. Packet simulation starts in phase 2–3."))
            .child(div().p_5().rounded_lg().border_1().border_color(rgb(0x344252)).flex().flex_col().gap_4()
                .child(div().text_sm().text_color(rgb(0x9ca9b8)).child("TOPOLOGY FIXTURE   /   select a node or link"))
                .child(topology))
            .child(div().flex().gap_5().flex_1()
                .child(div().flex_1().p_5().rounded_lg().bg(rgb(0x16212c)).flex().flex_col().gap_4()
                    .child(div().text_sm().text_color(rgb(0x9ca9b8)).child("COPIED BYTE BUFFER   /   select a byte"))
                    .child(hex)
                    .child(div().text_sm().text_color(rgb(0x9ca9b8)).child("UTF-8 fixture: Wirebox / Zig to Rust / ABI v1"))
                    .child(div().id("read-probe").px_4().py_3().rounded_md().bg(rgb(0x275d4e)).cursor_pointer()
                        .child("Read Zig batch  ·  Ctrl+R")
                        .on_click(cx.listener(|this, _, window, cx| this.read_probe(&ReadProbe, window, cx)))))
                .child(div().w(px(300.)).p_5().rounded_lg().bg(rgb(0x16212c)).flex().flex_col().gap_4()
                    .child(div().text_sm().text_color(rgb(0x9ca9b8)).child("INSPECTOR"))
                    .child(div().text_xl().child(SharedString::from(self.selected.clone())))
                    .child(div().text_sm().child(SharedString::from(self.detail.clone())))
                    .child(div().text_sm().text_color(rgb(0x7fd6b9)).child(format!("Event {} / {} bytes / copied", self.event.id, self.event.bytes.len())))
                    .child(div().text_sm().text_color(rgb(0x9ca9b8)).child("Probe events are transport records, not scheduled network events."))))
            .child(div().text_xs().text_color(rgb(0x9ca9b8))
                .child("Zig 0.17.0 + Rust 1.99.1 + GPUI 0.2.2   |   Esc: reset selection   |   Ctrl+Q: close"))
    }
}

fn main() {
    let log = SmokeLog(std::env::var_os("WIREBOX_UI_LOG").map(PathBuf::from));
    Application::new().run(move |cx: &mut App| {
        cx.bind_keys([
            KeyBinding::new("ctrl-q", CloseWindow, None),
            KeyBinding::new("ctrl-r", ReadProbe, None),
            KeyBinding::new("escape", ClearSelection, None),
        ]);
        let close_log = log.clone();
        cx.on_window_closed(move |cx| {
            close_log.write("window_closed");
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();
        let bounds = Bounds::centered(None, size(px(1120.), px(800.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_min_size: Some(size(px(1000.), px(800.))),
                ..Default::default()
            },
            |window, cx| {
                window.set_window_title("Wirebox · Phase 1");
                cx.new(|cx| Studio::new(window, cx, log))
            },
        )
        .expect("open Wirebox window");
        cx.activate(true);
    });
}
