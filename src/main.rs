#![allow(clippy::many_single_char_names)]
use gtk::*;
use serde::Deserialize;

const URL: &str = "http://91.132.145.114/json/stations/search?";

fn main() {
    gtk::init().unwrap();

    //ureq
    let mut client = ureq::agent();
    client.set("User-Agent", "nanowave(https://github.com/sigmaSd/nanowave)");

    // Main window
    let w = Window::new(WindowType::Toplevel);
    let v = Box::new(Orientation::Vertical, 10);
    let h = Box::new(Orientation::Horizontal, 10);
    let l = Label::new(Some("Search: "));
    let e = Entry::new();
    let g = Grid::new();
    let sw: ScrolledWindow = ScrolledWindow::new::<Adjustment, Adjustment>(None, None);

    // Playing winow
    let grid = g.clone();
    let grid2 = g.clone();
    let row = h.clone();
    let row2 = h.clone();
    let hb = Box::new(Orientation::Horizontal, 10);
    let label = Label::new(Some("Playing station!"));
    let ret = Button::with_label("Return");
    let hbc = hb.clone();
    let hbc2 = hb.clone();
    ret.connect_clicked(move |_| {
        hbc2.hide();
        grid2.show();
        row2.show();
    });
    hb.pack_start(&label, false, false, 10);
    hb.pack_start(&Label::new(None), true, true, 10);
    hb.pack_start(&ret, false, false, 10);

    // Signals
    e.connect_activate(move |entry| {
        //clear
        grid.foreach(|c| grid.remove(c));
        let text = entry.get_text();
        let text = text.to_string();
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
            let row = row.clone();
            let grid = grid.clone();
            let grid2 = grid.clone();
            let hbc = hbc.clone();
            btn.connect_clicked(move |_| {
                row.hide();
                grid2.hide();
                hbc.show_all();

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

            grid.attach(&btn, pos.0, pos.1, 1, 1);
            pos.0 += 1;
            if pos.0 == width {
                pos.0 = 0;
                pos.1 += 1;
            }
        }
        grid.show_all();
    });

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
