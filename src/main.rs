#![allow(clippy::many_single_char_names)]
use gtk::*;
use serde::Deserialize;

const URL: &str = "http://91.132.145.114/json/stations/search?";

macro_rules! shadow_clone {
    ($($i:ident),+) => ($(let $i = $i.clone();)+)
}

fn main() {
    gtk::init().unwrap();

    //ureq
    let mut client = ureq::agent();
    client.set(
        "User-Agent",
        "nanowave(https://github.com/sigmaSd/nanowave)",
    );

    // Main window
    let w = Window::new(WindowType::Toplevel);
    let v = Box::new(Orientation::Vertical, 10);
    let h = Box::new(Orientation::Horizontal, 10);
    let l = Label::new(Some("Search: "));
    let e = Entry::new();
    let g = Grid::new();
    let sw: ScrolledWindow = ScrolledWindow::new::<Adjustment, Adjustment>(None, None);

    // Playing winow
    let hb = Box::new(Orientation::Horizontal, 10);
    let label = Label::new(Some("Playing station!"));
    let ret = Button::with_label("Return");
    hb.pack_start(&label, false, false, 10);
    hb.pack_start(&Label::new(None), true, true, 10);
    hb.pack_start(&ret, false, false, 10);

    // Signals
    {
        shadow_clone!(hb, g, h);
        ret.connect_clicked(move |_| {
            hb.hide();
            g.show();
            h.show();
        });
    }
    {
        shadow_clone!(hb, g, h);
        e.connect_activate(move |entry| {
            //clear
            g.foreach(|c| g.remove(c));
            let text = entry.get_text().to_string();
            let text = text.trim();

            let url = if text.contains("c:") {
                let country = text.split("c:").nth(1).unwrap();
                let mut country = country.chars();
                let mut c = country.next().unwrap().to_uppercase().to_string();
                let cc: String = country.collect();
                c.push_str(&cc);

                format!("{}country={}", URL, c)
            } else {
                format!("{}name={}", URL, text)
            };

            let stations: Vec<Station> = client.get(&url).call().into_json_deserialize().unwrap();

            let width = 5;
            let mut pos = (0, 0);

            for s in stations.into_iter() {
                let btn = Button::with_label(&s.name);
                g.attach(&btn, pos.0, pos.1, 1, 1);
                shadow_clone!(h, g, hb);
                btn.connect_clicked(move |_| {
                    h.hide();
                    g.hide();
                    hb.show_all();

                    std::process::Command::new("pkill")
                        .arg("mpv")
                        .spawn()
                        .unwrap()
                        .wait()
                        .unwrap();

                    std::process::Command::new("mpv")
                        .arg(&s.url)
                        .spawn()
                        .unwrap();
                });

                pos.0 += 1;
                if pos.0 == width {
                    pos.0 = 0;
                    pos.1 += 1;
                }
            }
            g.show_all();
        });
    }

    // bind
    h.add(&l);
    h.add(&e);
    v.add(&hb);
    v.add(&h);
    v.add(&g);
    sw.add(&v);
    w.add(&sw);

    // show main winow
    // hide playing window
    w.show_all();
    hb.hide();

    //exit
    w.connect_delete_event(move |_, _| {
        std::process::Command::new("pkill")
            .arg("mpv")
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
        main_quit();
        Inhibit(false)
    });

    gtk::main();
}

#[derive(Debug, Deserialize)]
struct Station {
    name: String,
    url: String,
}
