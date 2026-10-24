mod model;

use gpui::{
    App, Application, Bounds, Context, FocusHandle, KeyBinding, KeyDownEvent, PathPromptOptions,
    SharedString, Window, WindowBounds, WindowOptions, actions, div, prelude::*, px, rgb, size,
};
use model::{Config, Model, covers, field_description, outcome_name};
use std::{
    fs::OpenOptions,
    io::{Read, Write},
    path::PathBuf,
    time::{Duration, Instant},
};
use wyrlex_kernel::network::{field_name, kind_name, reason_name};

actions!(
    wyrlex,
    [
        CloseWindow,
        RunSimulation,
        PauseSimulation,
        StepSimulation,
        ResetSimulation,
        SaveSession,
        OpenSession,
        ClearSelection
    ]
);

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
#[derive(Clone)]
enum EditKind {
    Address(String),
    Destination,
    Ttl,
    Gateway,
    Bitrate,
    Propagation,
    Routes,
    DisabledPorts,
    Links,
}
#[derive(Clone)]
struct Edit {
    kind: EditKind,
    label: String,
    value: String,
    replace: bool,
}
struct Studio {
    focus: FocusHandle,
    model: Option<Model>,
    draft: Option<Config>,
    running: bool,
    slow_run: bool,
    epoch: u64,
    selected_trace: Option<u64>,
    selected_field: Option<usize>,
    selected_byte: Option<usize>,
    before: bool,
    filter: Option<String>,
    zoom: f32,
    trace_page: usize,
    edit: Option<Edit>,
    message: String,
    log: SmokeLog,
    history: Option<model::Frame>,
    measurement: Option<(Instant, String)>,
}
impl Studio {
    fn new(window: &mut Window, cx: &mut Context<Self>, log: SmokeLog) -> Self {
        let focus = cx.focus_handle();
        focus.focus(window);
        let model = Config::preset("S01").and_then(Model::new);
        let message = model
            .as_ref()
            .err()
            .cloned()
            .unwrap_or_else(|| "Siap. Run atau Step untuk mengirim ping.".into());
        log.write("opened:network-v1");
        let history = std::env::var_os("WYRLEX_ACCEPTANCE")
            .map(|_| model::acceptance_history().expect("acceptance history"));
        if history.is_some() {
            cx.spawn(async move |this,cx| {
                for pulse in 0..1800u64 {
                    cx.background_executor().timer(Duration::from_secs(1)).await;
                    if !this.update(cx,|this,cx| {
                        let start=Instant::now();
                        let result: model::Result<()> = (|| {
                            if let Some(m)=&mut this.model {
                                if m.finished() { *m=Model::new(m.config.clone())?; }
                                m.advance(1)?;
                                if pulse%10==0 { let saved=m.save()?; let loaded=Model::load(saved.as_bytes())?; if loaded.report()!=m.report(){return Err("soak roundtrip".into());} }
                            }
                            Ok(())
                        })();
                        if let Err(e)=result {this.log.write(&format!("soak_failure:{pulse}:{e}"));return false;}
                        if pulse%2==0 {this.pause();}
                        if let Some(h)=&this.history {
                            let index=(pulse as usize*137)%h.trace.len();
                            this.selected_trace=Some(h.trace[index].ordinal);this.trace_page=index/50;
                            this.selected_field=None;this.selected_byte=None;this.filter=None;
                        }
                        this.measurement=Some((start,format!("scripted:{}:{}",if pulse%2==0{"pause"}else{"selection"},pulse)));
                        this.message=format!("ACCEPTANCE · 160 independent runs / 10000 retained rows · pulse {pulse}/1800");
                        if pulse==1799{this.log.write("soak_complete:1800");}
                        cx.notify();true
                    }).unwrap_or(false) {break;}
                }
            }).detach();
        }
        Self {
            focus,
            model: model.ok(),
            draft: None,
            running: false,
            slow_run: false,
            epoch: 0,
            selected_trace: None,
            selected_field: None,
            selected_byte: None,
            before: false,
            filter: None,
            zoom: 1.,
            trace_page: 0,
            edit: None,
            message,
            log,
            history,
            measurement: None,
        }
    }
    fn pause(&mut self) {
        self.running = false;
        // Invalidate every detached continuation before replacing/resetting an owner.
        self.epoch += 1;
    }
    fn replace(&mut self, model: Model) {
        self.pause();
        self.model = Some(model);
        self.history = None;
        self.draft = None;
        self.clear();
        self.edit = None;
    }
    fn clear(&mut self) {
        self.trace_page = 0;
        self.selected_trace = None;
        self.selected_field = None;
        self.selected_byte = None;
        self.before = false;
        self.filter = None;
    }
    fn preset(&mut self, case: &str, cx: &mut Context<Self>) {
        self.pause();
        match Config::preset(case).and_then(Model::new) {
            Ok(model) => {
                self.replace(model);
                self.message = format!("{case} siap; belum dieksekusi.");
                self.log.write(&format!("preset:{case}"));
            }
            Err(e) => self.message = e,
        }
        cx.notify();
    }
    fn advance(&mut self, budget: u64, cx: &mut Context<Self>) {
        let Some(model) = &mut self.model else { return };
        match model.advance(budget) {
            Ok(()) => {
                let s = model.frame.summary;
                self.message = format!(
                    "{} · {} · {} event dieksekusi · {} row trace · {} pending",
                    outcome_name(s.outcome),
                    reason_name(s.reason),
                    s.executed,
                    s.trace_total,
                    s.pending_events
                );
                if self.selected_trace.is_none() {
                    self.selected_trace = model
                        .frame
                        .trace
                        .iter()
                        .rev()
                        .find(|t| t.version != 0)
                        .map(|t| t.ordinal);
                }
                self.log.write(&format!(
                    "batch:budget={budget}:executed={}:outcome={}:pending={}",
                    s.executed, s.outcome, s.pending_events
                ));
                if model.finished() {
                    self.pause();
                }
            }
            Err(error) => {
                self.message = error;
                self.pause();
                self.log.write("run_error");
            }
        }
        cx.notify();
    }
    fn run(&mut self, _: &RunSimulation, _: &mut Window, cx: &mut Context<Self>) {
        if self.running || self.model.as_ref().is_none_or(Model::finished) {
            return;
        }
        self.edit = None;
        self.running = true;
        self.epoch += 1;
        let epoch = self.epoch;
        // Playback pacing yields to input; neither mode changes virtual time.
        let (delay_ms, budget) = if self.slow_run { (1000, 1) } else { (40, 4) };
        self.log.write("run");
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(delay_ms))
                    .await;
                let keep_running = this
                    .update(cx, |this, cx| {
                        if !this.running || this.epoch != epoch {
                            return false;
                        }
                        this.advance(budget, cx);
                        this.running && this.epoch == epoch
                    })
                    .unwrap_or(false);
                if !keep_running {
                    break;
                }
            }
        })
        .detach();
        cx.notify();
    }
    fn step(&mut self, _: &StepSimulation, _: &mut Window, cx: &mut Context<Self>) {
        self.pause();
        self.edit = None;
        self.advance(1, cx);
    }
    fn reset(&mut self, _: &ResetSimulation, _: &mut Window, cx: &mut Context<Self>) {
        self.pause();
        if let Some(model) = &self.model {
            let config = model.config.clone();
            let baseline = model.baseline.clone();
            match Model::new(config) {
                Ok(mut model) => {
                    model.baseline = baseline;
                    self.replace(model);
                    self.message = "Reset ke konfigurasi aktif; cache dan antrean baru.".into();
                    self.log.write("reset");
                }
                Err(e) => self.message = e,
            }
        }
        cx.notify();
    }
    fn branch(&mut self, config: Config, cx: &mut Context<Self>) {
        self.pause();
        self.draft = Some(config);
        self.message = "Draft tersimpan. Apply draft untuk validasi dan membuat cabang.".into();
        cx.notify();
    }
    fn apply_draft(&mut self, cx: &mut Context<Self>) {
        let Some(config) = self.draft.clone() else {
            return;
        };
        self.pause();
        if let Some(source) = &self.model {
            match source.branch(config) {
                Ok(model) => {
                    self.replace(model);
                    self.message="Cabang konfigurasi baru siap. Hasil asal tersimpan sebagai konteks perbandingan.".into();
                    self.log.write("branch");
                }
                Err(e) => {
                    self.message = format!("Konfigurasi ditolak; hasil asal tetap tersedia: {e}");
                    self.log.write("branch_rejected");
                }
            }
        }
        cx.notify();
    }
    fn begin_edit(&mut self, kind: EditKind, cx: &mut Context<Self>) {
        self.pause();
        if let Some(model) = &self.model {
            let c = self.draft.as_ref().unwrap_or(&model.config);
            let (label, value) = match &kind {
                EditKind::Address(port) => {
                    (format!("IPv4/prefix {port}"), c.addresses[port].clone())
                }
                EditKind::Destination => ("Tujuan IPv4".into(), c.destination.clone()),
                EditKind::Ttl => ("TTL 1..255".into(), c.ttl.to_string()),
                EditKind::Gateway => (
                    "Gateway route R1 198.51.100.2/32 via eth1".into(),
                    c.route_via.clone().unwrap_or_else(|| "10.0.1.2".into()),
                ),
                EditKind::Bitrate => ("Link bitrate (bps)".into(), c.bitrate.to_string()),
                EditKind::Routes => (
                    "Routes: device prefix port gateway; ... (kosong = hapus semua)".into(),
                    c.route_text(),
                ),
                EditKind::DisabledPorts => (
                    "Port down: PC1:eth0 SW1:p0 SW1:p1 R1:eth0 R1:eth1 PC2:eth0".into(),
                    c.disabled_ports.clone(),
                ),
                EditKind::Links => (
                    "Link: L1 bitrate_bps delay_ns; L2 ...; L3 ...".into(),
                    c.links.clone(),
                ),
                EditKind::Propagation => {
                    ("Link propagation (ns)".into(), c.propagation.to_string())
                }
            };
            self.edit = Some(Edit {
                kind,
                label,
                value,
                replace: true,
            });
        }
        cx.notify();
    }
    fn commit_edit(&mut self, cx: &mut Context<Self>) {
        let Some(edit) = self.edit.take() else { return };
        let Some(model) = &self.model else { return };
        let mut config = self.draft.as_ref().unwrap_or(&model.config).clone();
        let retained = edit.clone();
        let parsed: std::result::Result<(), String> = (|| {
            match edit.kind {
                EditKind::Address(port) => {
                    config.addresses.insert(port, edit.value);
                }
                EditKind::Routes => config.routes = Some(edit.value),
                EditKind::DisabledPorts => config.disabled_ports = edit.value,
                EditKind::Links => config.links = edit.value,
                EditKind::Destination => config.destination = edit.value,
                EditKind::Gateway => config.set_alias_route(Some(edit.value)),
                EditKind::Ttl => {
                    config.ttl = edit.value.parse::<u32>().map_err(|e| e.to_string())?
                }
                EditKind::Bitrate => {
                    config.bitrate = edit.value.parse::<u64>().map_err(|e| e.to_string())?
                }
                EditKind::Propagation => {
                    config.propagation = edit.value.parse::<u64>().map_err(|e| e.to_string())?
                }
            }
            Ok(())
        })();
        match parsed {
            Ok(()) => self.branch(config, cx),
            Err(e) => {
                self.edit = Some(retained);
                self.message = e;
                cx.notify();
            }
        }
    }
    fn key(&mut self, event: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        if self.edit.is_none() {
            return;
        }
        let key = &event.keystroke;
        match key.key.as_str() {
            "enter" => self.commit_edit(cx),
            "escape" => self.edit = None,
            "backspace" | "delete" => {
                let edit = self.edit.as_mut().unwrap();
                if edit.replace {
                    edit.value.clear();
                    edit.replace = false;
                } else {
                    edit.value.pop();
                }
            }
            "a" if key.modifiers.control => self.edit.as_mut().unwrap().replace = true,
            "v" if key.modifiers.control => {
                if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
                    self.append_edit(&text);
                }
            }
            _ => {
                if !key.modifiers.control
                    && !key.modifiers.alt
                    && let Some(text) = &key.key_char
                {
                    self.append_edit(text);
                }
            }
        }
        cx.stop_propagation();
        cx.notify();
    }
    fn append_edit(&mut self, text: &str) {
        if text.len() > 4096 || !text.chars().all(|c| c.is_ascii_graphic() || c == ' ') {
            return;
        }
        let edit = self.edit.as_mut().unwrap();
        if edit.replace {
            edit.value.clear();
            edit.replace = false;
        }
        if edit.value.len() + text.len() <= 4096 {
            edit.value.push_str(text);
        }
    }
    fn save(&mut self, _: &SaveSession, _: &mut Window, cx: &mut Context<Self>) {
        if self.history.is_some() {
            self.message =
                "Acceptance history is read-only; reset to save a normal session.".into();
            cx.notify();
            return;
        }
        self.pause();
        let Some(model) = &self.model else { return };
        let payload = match model.save() {
            Ok(p) => p,
            Err(e) => {
                self.message = e;
                cx.notify();
                return;
            }
        };
        let directory = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let answer = cx.prompt_for_new_path(&directory, Some("wyrlex-session.wb.json"));
        cx.spawn(async move |this, cx| {
            let result = match answer.await {
                Ok(Ok(Some(path))) => {
                    cx.background_executor()
                        .spawn(async move {
                            std::fs::write(&path, payload)
                                .map(|_| format!("Tersimpan: {}", path.display()))
                                .map_err(|e| e.to_string())
                        })
                        .await
                }
                Ok(Ok(None)) => Ok("Simpan dibatalkan.".into()),
                other => Err(format!("Dialog simpan gagal: {other:?}")),
            };
            let _ = this.update(cx, |this, cx| {
                this.message = result.unwrap_or_else(|e| e);
                this.log.write(&this.message);
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }
    fn open(&mut self, _: &OpenSession, _: &mut Window, cx: &mut Context<Self>) {
        self.pause();
        let epoch = self.epoch;
        let answer = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some("Buka sesi Wyrlex (.wb.json)".into()),
        });
        cx.spawn(async move |this,cx| {
            let result=match answer.await {
                Ok(Ok(Some(paths))) if paths.len()==1 => {
                    let path=paths[0].clone();
                    cx.background_executor().spawn(async move {
                        let file=std::fs::File::open(path).map_err(|e|e.to_string())?;
                        let mut bytes=Vec::new();
                        file.take(model::FILE_LIMIT as u64+1).read_to_end(&mut bytes).map_err(|e|e.to_string())?;
                        Ok(Some(bytes))
                    }).await
                }
                Ok(Ok(None))=>Ok(None),
                other=>Err(format!("Dialog buka gagal: {other:?}")),
            };
            let _=this.update(cx,|this,cx| {
                if this.epoch!=epoch {return}
                match result {
                    Ok(Some(bytes))=>match Model::load(&bytes) {
                        Ok(model)=>{this.replace(model);this.message="Sesi dibuka; hasil diverifikasi ulang dari command log. Dalam keadaan pause.".into();this.log.write("loaded:verified");}
                        Err(e)=>{this.message=format!("File ditolak; sesi aktif tetap tersedia: {e}");this.log.write("load_rejected");}
                    },
                    Ok(None)=>this.message="Buka dibatalkan.".into(),Err(e)=>this.message=e,
                }
                cx.notify();
            });
        }).detach();
        cx.notify();
    }
}

fn button(id: &'static str, label: impl Into<SharedString>) -> gpui::Stateful<gpui::Div> {
    div()
        .id(id)
        .px_3()
        .py_2()
        .rounded_md()
        .bg(rgb(0x244638))
        .cursor_pointer()
        .text_sm()
        .child(label.into())
}
fn caption(label: impl Into<SharedString>) -> gpui::Div {
    div()
        .text_xs()
        .text_color(rgb(0x93a6b6))
        .child(label.into())
}
impl Render for Studio {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some((start, label)) = self.measurement.take() {
            let log = self.log.clone();
            window.on_next_frame(move |_, _| {
                log.write(&format!(
                    "latency_us:{label}:{}",
                    start.elapsed().as_micros()
                ))
            });
        }
        let mut root = div()
            .id("studio")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(Self::key))
            .on_action(|_: &CloseWindow, window, _| window.remove_window())
            .on_action(cx.listener(Self::run))
            .on_action(cx.listener(Self::step))
            .on_action(cx.listener(Self::reset))
            .on_action(cx.listener(Self::save))
            .on_action(cx.listener(Self::open))
            .on_action(cx.listener(|this, _: &PauseSimulation, _, cx| {
                this.pause();
                this.measurement = Some((Instant::now(), "native:pause".into()));
                this.log.write("pause");
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &ClearSelection, _, cx| {
                this.clear();
                this.edit = None;
                cx.notify();
            }))
            .size_full()
            .p_4()
            .flex()
            .flex_col()
            .gap_3()
            .font_family("Segoe UI")
            .bg(rgb(0x101820))
            .text_color(rgb(0xe6edf3));
        root = root.child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .child(div().text_2xl().child("Wyrlex"))
                        .child(caption("PHASE 4 · Packet Studio · synthetic simulation")),
                )
                .child(
                    div()
                        .flex()
                        .gap_2()
                        .child(
                            button(
                                "pace",
                                if self.slow_run {
                                    "Pace: slow"
                                } else {
                                    "Pace: fast"
                                },
                            )
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.pause();
                                this.slow_run = !this.slow_run;
                                this.log.write(if this.slow_run {
                                    "pace:slow"
                                } else {
                                    "pace:fast"
                                });
                                cx.notify();
                            })),
                        )
                        .child(
                            button(
                                "run",
                                if self.running {
                                    "Running…"
                                } else {
                                    "Run · Ctrl+R"
                                },
                            )
                            .on_click(
                                cx.listener(|this, _, w, cx| this.run(&RunSimulation, w, cx)),
                            ),
                        )
                        .child(button("pause", "Pause · Space").on_click(cx.listener(
                            |this, _, _, cx| {
                                this.pause();
                                this.log.write("pause");
                                cx.notify();
                            },
                        )))
                        .child(button("step", "Step · F10").on_click(
                            cx.listener(|this, _, w, cx| this.step(&StepSimulation, w, cx)),
                        ))
                        .child(button("reset", "Reset").on_click(
                            cx.listener(|this, _, w, cx| this.reset(&ResetSimulation, w, cx)),
                        ))
                        .child(
                            button("save", "Save").on_click(
                                cx.listener(|this, _, w, cx| this.save(&SaveSession, w, cx)),
                            ),
                        )
                        .child(button("open", "Open").on_click(
                            cx.listener(|this, _, w, cx| this.open(&OpenSession, w, cx)),
                        )),
                ),
        );
        let mut presets = div()
            .flex()
            .gap_2()
            .items_center()
            .child(caption("SCENARIO"));
        for (index, (case, label)) in [
            ("S01", "S01 Healthy"),
            ("S02", "S02 No route"),
            ("S03", "S03 TTL"),
            ("S04", "S04 ARP timeout"),
            ("S05", "S05 Invalid"),
        ]
        .into_iter()
        .enumerate()
        {
            presets = presets.child(
                div()
                    .id(("preset", index))
                    .px_3()
                    .py_2()
                    .rounded_md()
                    .text_sm()
                    .cursor_pointer()
                    .bg(rgb(
                        if self.model.as_ref().is_some_and(|m| m.config.case == case) {
                            0x30654e
                        } else {
                            0x1c2a38
                        },
                    ))
                    .child(label)
                    .on_click(cx.listener(move |this, _, _, cx| this.preset(case, cx))),
            );
        }
        root = root.child(presets);
        let Some(model) = &self.model else {
            return root.child(div().child(self.message.clone()));
        };
        let config = model.config.clone();
        let frame = self.history.as_ref().unwrap_or(&model.frame);
        let summary = frame.summary;
        root = root.child(
            div()
                .flex()
                .gap_2()
                .items_center()
                .child(
                    button("destination", format!("Ping → {}", config.destination)).on_click(
                        cx.listener(|this, _, _, cx| this.begin_edit(EditKind::Destination, cx)),
                    ),
                )
                .child(
                    button("ttl", format!("TTL {}", config.ttl))
                        .on_click(cx.listener(|this, _, _, cx| this.begin_edit(EditKind::Ttl, cx))),
                )
                .child(
                    button(
                        "route",
                        if config.route_via.is_some() {
                            "Edit R1 alias route"
                        } else {
                            "Add missing R1 route"
                        },
                    )
                    .on_click(cx.listener(|this, _, _, cx| this.begin_edit(EditKind::Gateway, cx))),
                )
                .child(
                    button("route-remove", "Remove R1 alias route").on_click(cx.listener(
                        |this, _, _, cx| {
                            if let Some(m) = &this.model {
                                let mut c = this.draft.as_ref().unwrap_or(&m.config).clone();
                                c.set_alias_route(None);
                                this.branch(c, cx);
                            }
                        },
                    )),
                )
                .child(
                    button(
                        "port",
                        if config.pc2_enabled {
                            "PC2 port: UP"
                        } else {
                            "PC2 port: DOWN"
                        },
                    )
                    .on_click(cx.listener(|this, _, _, cx| {
                        if let Some(m) = &this.model {
                            let mut c = this.draft.as_ref().unwrap_or(&m.config).clone();
                            c.pc2_enabled = !c.pc2_enabled;
                            this.branch(c, cx);
                        }
                    })),
                ),
        );
        if let Some(edit) = &self.edit {
            root = root.child(
                div()
                    .p_3()
                    .rounded_md()
                    .bg(rgb(0x293b35))
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(format!(
                        "{}: {}│  · ketik untuk mengganti · Backspace · Ctrl+V · Enter / Esc",
                        edit.label, edit.value
                    ))
                    .child(
                        button("apply", "Save to draft")
                            .on_click(cx.listener(|this, _, _, cx| this.commit_edit(cx))),
                    ),
            );
        }
        let model = self.model.as_ref().unwrap();
        let mut nodes = div().flex().items_center().gap_3().w(px(1080. * self.zoom));
        for (i, name) in ["PC1", "SW1", "R1", "PC2"].into_iter().enumerate() {
            if i > 0 {
                let link = ["L1", "L2", "L3"][i - 1];
                let (bitrate, delay) = config
                    .link_entries()
                    .unwrap_or_default()
                    .get(link)
                    .copied()
                    .unwrap_or((config.bitrate, config.propagation));
                nodes = nodes.child(
                    div()
                        .id(("link", i))
                        .w(px(100. * self.zoom))
                        .border_b_2()
                        .border_color(rgb(0x5b8291))
                        .py_2()
                        .cursor_pointer()
                        .child(caption(format!("{link} · {bitrate} bps · {delay} ns")))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.trace_page = 0;
                            this.filter = Some(link.into());
                            this.selected_trace = None;
                            this.selected_field = None;
                            this.selected_byte = None;
                            this.before = false;
                            this.log.write(&format!("filter:{link}"));
                            cx.notify();
                        })),
                );
            }
            let role = match name {
                "SW1" => "Ethernet switch",
                "R1" => "Static IPv4 router",
                _ => "IPv4 host",
            };
            let mut node = div()
                .id(("node", i))
                .w(px(180. * self.zoom))
                .p_3()
                .rounded_md()
                .border_1()
                .border_color(rgb(if self.filter.as_deref() == Some(name) {
                    0x79c9a1
                } else {
                    0x344959
                }))
                .bg(rgb(0x192c38))
                .cursor_pointer()
                .child(div().text_lg().child(name))
                .child(caption(role));
            for (port, address) in config.addresses.iter().filter(|(p, _)| p.starts_with(name)) {
                let port = port.clone();
                node = node.child(
                    div()
                        .id(SharedString::from(format!("address-{port}")))
                        .text_xs()
                        .py_1()
                        .child(format!("{address} · edit"))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            cx.stop_propagation();
                            this.begin_edit(EditKind::Address(port.clone()), cx);
                        })),
                );
            }
            nodes = nodes.child(node.on_click(cx.listener(move |this, _, _, cx| {
                this.trace_page = 0;
                this.filter = Some(name.into());
                this.selected_trace = None;
                this.selected_field = None;
                this.selected_byte = None;
                this.before = false;
                this.log.write(&format!("filter:{name}"));
                cx.notify();
            })));
        }
        root = root.child(
            div()
                .flex()
                .gap_2()
                .items_center()
                .child(
                    div()
                        .id("topology")
                        .flex_1()
                        .min_w_0()
                        .overflow_x_scroll()
                        .child(nodes),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(
                            button("zoom-in", "+").on_click(cx.listener(|this, _, _, cx| {
                                this.zoom = (this.zoom + 0.1).min(1.6);
                                cx.notify();
                            })),
                        )
                        .child(
                            button("zoom-out", "−").on_click(cx.listener(|this, _, _, cx| {
                                this.zoom = (this.zoom - 0.1).max(0.7);
                                cx.notify();
                            })),
                        ),
                ),
        );
        root=root.child(div().flex().justify_between()
            .child(caption(format!("t = {} ns · {} scheduler events · {} trace rows · {} packet versions · dropped trace {}",
                summary.now_ns,summary.executed,summary.trace_total,summary.packet_versions,summary.trace_dropped)))
            .child(div().flex().gap_2()
                .child(button("bitrate","Bitrate").on_click(cx.listener(|this,_,_,cx|this.begin_edit(EditKind::Bitrate,cx))))
                .child(button("propagation",format!("Delay {} ns",config.propagation)).on_click(cx.listener(|this,_,_,cx|this.begin_edit(EditKind::Propagation,cx))))));
        root =
            root.child(
                div()
                    .flex()
                    .gap_2()
                    .child(button("routes-editor", "Routes").on_click(
                        cx.listener(|this, _, _, cx| this.begin_edit(EditKind::Routes, cx)),
                    ))
                    .child(button("ports-editor", "Port down").on_click(
                        cx.listener(|this, _, _, cx| this.begin_edit(EditKind::DisabledPorts, cx)),
                    ))
                    .child(button("links-editor", "Per-link timing").on_click(
                        cx.listener(|this, _, _, cx| this.begin_edit(EditKind::Links, cx)),
                    ))
                    .child(
                        button(
                            "apply-draft",
                            if self.draft.is_some() {
                                "Apply draft *"
                            } else {
                                "Apply draft"
                            },
                        )
                        .on_click(cx.listener(|this, _, _, cx| this.apply_draft(cx))),
                    )
                    .child(
                        button("discard-draft", "Discard draft").on_click(cx.listener(
                            |this, _, _, cx| {
                                this.draft = None;
                                this.edit = None;
                                this.message = "Draft dibuang; konfigurasi aktif tetap.".into();
                                cx.notify();
                            },
                        )),
                    ),
            );
        if let Some(baseline) = &model.baseline {
            root=root.child(caption(format!("PERBANDINGAN · sebelum: {} / {} · sekarang: {} / {} · cabang selalu dimulai dengan cache kosong",
                outcome_name(baseline["report"]["summary"]["outcome"].as_u64().unwrap_or(0) as u32),
                reason_name(baseline["report"]["summary"]["reason"].as_u64().unwrap_or(0) as u32),outcome_name(summary.outcome),reason_name(summary.reason))));
        }
        let mut rows = div()
            .id("trace-scroll")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .flex()
            .flex_col()
            .gap_1();
        let mut visible_index = 0usize;
        for trace in &frame.trace {
            let entity = model.entity_name(trace.entity);
            let visible = self.filter.as_ref().is_none_or(|f| {
                entity == f
                    || entity.starts_with(&format!("{f}:"))
                    || model
                        .frame
                        .transmissions
                        .get(&trace.transmission)
                        .is_some_and(|tx| model.entity_name(tx.link) == f)
            });
            if !visible {
                continue;
            }
            let page = visible_index / 50;
            visible_index += 1;
            if page != self.trace_page {
                continue;
            }
            let ordinal = trace.ordinal;
            rows = rows.child(
                div()
                    .id(("trace", ordinal as usize))
                    .p_2()
                    .rounded_sm()
                    .cursor_pointer()
                    .bg(rgb(if self.selected_trace == Some(ordinal) {
                        0x2b5144
                    } else {
                        0x192733
                    }))
                    .child(
                        div()
                            .text_xs()
                            .child(format!("#{ordinal} · {} ns · {entity}", trace.time_ns)),
                    )
                    .child(div().text_sm().child(format!(
                        "{} · {} · v{}",
                        kind_name(trace.kind),
                        reason_name(trace.reason),
                        trace.version
                    )))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.measurement = Some((Instant::now(), "native:selection".into()));
                        this.selected_trace = Some(ordinal);
                        this.before = false;
                        this.selected_field = None;
                        this.selected_byte = None;
                        this.log.write(&format!("trace:{ordinal}"));
                        cx.notify();
                    })),
            );
        }
        let timeline = div()
            .w(px(390.))
            .flex()
            .flex_col()
            .min_h_0()
            .p_3()
            .gap_2()
            .rounded_md()
            .bg(rgb(0x14212d))
            .child(
                div()
                    .flex()
                    .justify_between()
                    .child(caption(format!(
                        "TRACE · {}",
                        self.filter.as_deref().unwrap_or("all entities")
                    )))
                    .child(button("clear-filter", "All").on_click(cx.listener(
                        |this, _, _, cx| {
                            this.trace_page = 0;
                            this.filter = None;
                            cx.notify();
                        },
                    ))),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(button("trace-prev", "Previous").on_click(cx.listener(
                        |this, _, _, cx| {
                            this.trace_page = this.trace_page.saturating_sub(1);
                            cx.notify();
                        },
                    )))
                    .child(caption(format!(
                        "Page {} / {}",
                        self.trace_page + 1,
                        visible_index.div_ceil(50).max(1)
                    )))
                    .child(button("trace-next", "Next").on_click(cx.listener(
                        move |this, _, _, cx| {
                            this.trace_page =
                                (this.trace_page + 1).min(visible_index.saturating_sub(1) / 50);
                            cx.notify();
                        },
                    ))),
            )
            .child(rows);
        let trace = self
            .selected_trace
            .and_then(|id| frame.trace.iter().find(|t| t.ordinal == id));
        let version = trace.map_or(0, |t| if self.before { t.prior } else { t.version });
        let mut inspector = div()
            .flex_1()
            .min_w_0()
            .min_h_0()
            .flex()
            .flex_col()
            .p_3()
            .gap_2()
            .rounded_md()
            .bg(rgb(0x14212d));
        inspector = inspector.child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(caption("PACKET INSPECTOR · field ↔ bytes"))
                .child(
                    div()
                        .flex()
                        .gap_2()
                        .child(
                            button("before", if self.before { "● Before" } else { "Before" })
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.before = true;
                                    this.selected_field = None;
                                    this.selected_byte = None;
                                    cx.notify();
                                })),
                        )
                        .child(
                            button("after", if !self.before { "● After" } else { "After" })
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.before = false;
                                    this.selected_field = None;
                                    this.selected_byte = None;
                                    cx.notify();
                                })),
                        ),
                ),
        );
        if let Some(t) = trace {
            inspector = inspector.child(caption(format!(
                "row #{} · event {} ← cause {} · packet {} · v{} → v{} · detail {}/{}",
                t.ordinal,
                t.event_id,
                t.cause_id,
                t.packet_id,
                t.prior,
                t.version,
                t.detail_a,
                t.detail_b
            )));
        }
        if let Some(view) = frame.packets.get(&version) {
            inspector = inspector.child(caption(format!(
                "v{} · parent v{} · packet {} · {} bytes · layer {} · copied immutable bytes",
                view.packet.version_id,
                view.packet.parent_version,
                view.packet.packet_id,
                view.packet.bytes.len(),
                view.packet.layer
            )));
            if let Some(error) = &view.decode_error {
                inspector = inspector.child(caption(format!(
                    "Raw bytes tetap tersedia; field decode: {error}"
                )));
            }
            let mut fields = div()
                .id("fields-scroll")
                .w(px(290.))
                .min_h_0()
                .overflow_y_scroll()
                .flex()
                .flex_col()
                .gap_1();
            let mut group = "";
            for (index, field) in view.fields.iter().enumerate() {
                let name = field_name(field.code);
                let protocol = name.split('.').next().unwrap_or(name);
                if protocol != group {
                    fields = fields.child(caption(protocol.to_uppercase()));
                    group = protocol;
                }
                let active = self.selected_field == Some(index)
                    || self.selected_byte.is_some_and(|b| covers(field, b));
                let value = if field.bit_width <= 64 {
                    format!("{}", field.value)
                } else {
                    "raw".into()
                };
                fields = fields.child(
                    div()
                        .id(("field", index))
                        .p_2()
                        .rounded_sm()
                        .cursor_pointer()
                        .bg(rgb(if active { 0x2d5a49 } else { 0x1a2a37 }))
                        .child(div().text_xs().child(format!("{name} = {value}")))
                        .child(caption(format!(
                            "byte {} bit {} · {} bits",
                            field.byte_offset, field.bit_offset, field.bit_width
                        )))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.selected_field = Some(index);
                            this.selected_byte = None;
                            this.log.write(&format!("field:{name}"));
                            cx.notify();
                        })),
                );
            }
            let prior = trace.and_then(|t| model.frame.packets.get(&t.prior));
            let mut hex = div()
                .id("hex-scroll")
                .flex_1()
                .min_w_0()
                .min_h_0()
                .overflow_y_scroll()
                .flex()
                .flex_col()
                .gap_1();
            for (row, bytes) in view.packet.bytes.chunks(8).enumerate() {
                let mut line = div()
                    .flex()
                    .gap_1()
                    .items_center()
                    .child(caption(format!("{:04x}", row * 8)));
                for (column, byte) in bytes.iter().copied().enumerate() {
                    let offset = row * 8 + column;
                    let active = self.selected_byte == Some(offset)
                        || self
                            .selected_field
                            .and_then(|i| view.fields.get(i))
                            .is_some_and(|f| covers(f, offset));
                    let changed = !self.before
                        && prior.is_some_and(|p| {
                            p.packet.layer == view.packet.layer
                                && p.packet.bytes.get(offset) != Some(&byte)
                        });
                    line = line.child(
                        div()
                            .id(("byte", offset))
                            .w(px(32.))
                            .h(px(29.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded_sm()
                            .cursor_pointer()
                            .bg(rgb(if active {
                                0x357556
                            } else if changed {
                                0x654a28
                            } else {
                                0x203442
                            }))
                            .text_sm()
                            .child(format!("{byte:02x}"))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.selected_byte = Some(offset);
                                this.selected_field = None;
                                this.log.write(&format!("byte:{offset}={byte:02x}"));
                                cx.notify();
                            })),
                    );
                }
                hex = hex.child(line);
            }
            inspector = inspector.child(
                div()
                    .flex()
                    .flex_1()
                    .min_h_0()
                    .gap_3()
                    .child(fields)
                    .child(hex),
            );
            let detail = if let Some(field) = self.selected_field.and_then(|i| view.fields.get(i)) {
                field_description(field)
            } else if let Some(byte) = self.selected_byte {
                let names = view
                    .fields
                    .iter()
                    .filter(|f| covers(f, byte))
                    .map(|f| field_name(f.code))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("byte {byte} · {} · {names}", view.packet.bytes[byte])
            } else {
                "Klik field atau byte. Hijau: selection · amber: byte berubah dari prior pada layer yang sama.".into()
            };
            inspector = inspector.child(caption(detail));
        } else {
            inspector=inspector.child(caption("Pilih row trace yang memiliki versi paket. Row tanpa paket tidak menampilkan byte dari selection sebelumnya."));
        }
        root.child(div().flex().flex_1().min_h_0().gap_3().child(timeline).child(inspector))
            .child(div().text_sm().text_color(rgb(if summary.failed!=0{0xf2a095}else{0xa3dfbc})).child(self.message.clone()))
            .child(caption("Batas model: Ethernet / ARP / IPv4 / ICMP statis · tanpa hardware/vendor · scroll topologi untuk pan, +/− untuk zoom · Ctrl+S save / Ctrl+O open · Ctrl+Q close"))
    }
}
impl Drop for Studio {
    fn drop(&mut self) {
        self.pause();
        drop(self.model.take());
        self.log.write("network_destroyed");
    }
}
fn main() {
    let log = SmokeLog(std::env::var_os("WYRLEX_UI_LOG").map(PathBuf::from));
    Application::new().run(move |cx: &mut App| {
        cx.bind_keys([
            KeyBinding::new("ctrl-q", CloseWindow, None),
            KeyBinding::new("ctrl-r", RunSimulation, None),
            KeyBinding::new("space", PauseSimulation, None),
            KeyBinding::new("f10", StepSimulation, None),
            KeyBinding::new("ctrl-n", ResetSimulation, None),
            KeyBinding::new("ctrl-s", SaveSession, None),
            KeyBinding::new("ctrl-o", OpenSession, None),
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
        let bounds = Bounds::centered(None, size(px(1360.), px(940.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_min_size: Some(size(px(1160.), px(800.))),
                ..Default::default()
            },
            |window, cx| {
                window.set_window_title(if std::env::var_os("WYRLEX_ACCEPTANCE").is_some() {
                    "Wyrlex · Acceptance 10000 rows"
                } else {
                    "Wyrlex · Packet Studio"
                });
                cx.new(|cx| Studio::new(window, cx, log))
            },
        )
        .expect("open Wyrlex window");
        cx.activate(true);
    });
}
