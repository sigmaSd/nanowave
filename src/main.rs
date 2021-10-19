#![allow(clippy::many_single_char_names)]
use gtk::prelude::*;
use gtk::*;
use serde::Deserialize;
use std::io::Write;

macro_rules! shadow_clone {
    ($($i:ident),+) => ($(let $i = $i.clone();)+)
}

fn main() {
    gtk::init().unwrap();

    let path = dirs_next::cache_dir().unwrap().join("nanowave");
    let connection = if path.join("db.sqlite").exists() {
        sqlite::open(path.join("db.sqlite")).unwrap()
    } else {
        create_db(&path)
    };

    // Main window
    let w = Window::new(WindowType::Toplevel);
    w.maximize();
    let v = Box::new(Orientation::Vertical, 10);
    let h = Box::new(Orientation::Horizontal, 10);
    let l = Label::new(Some("Search: "));
    let e = Entry::new();
    let g = Grid::new();
    let search_tag = ComboBoxText::new();
    search_tag.append(None, "country");
    search_tag.append(None, "name");
    search_tag.set_active(Some(0));
    let sw: ScrolledWindow = ScrolledWindow::new::<Adjustment, Adjustment>(None, None);

    // Signals
    {
        shadow_clone!(g, search_tag);
        e.connect_activate(move |entry| {
            //clear
            g.foreach(|c| g.remove(c));
            let text = entry.text().to_string();
            let text = text.trim();

            let mut query = "SELECT * from mytable ".to_string();
            query.push_str("WHERE ");
            match search_tag.active_text().unwrap().to_string().as_str() {
                "name" => {
                    query.push_str(&format!("Name LIKE \"%{}%\"", text));
                }
                "country" => {
                    query.push_str(&format!("Country LIKE \"%{}%\"", text));
                }
                _ => unreachable!(),
            }

            let mut stations = vec![];
            let mut limit = 100;
            let _ = connection.iterate(query, |cols| {
                let mut cols = cols.iter();
                let name = cols.next().unwrap().1.unwrap().to_string();
                let country = cols.next().unwrap().1.unwrap().to_string();
                let url = cols.next().unwrap().1.unwrap().to_string();
                stations.push(Station { name, country, url });

                limit -= 1;

                limit != 0
            });

            let width = 5;
            let mut pos = (0, 0);

            for s in stations.into_iter() {
                let btn = Button::with_label(&s.name);
                g.attach(&btn, pos.0, pos.1, 1, 1);
                btn.connect_clicked(move |_| {
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
    h.add(&search_tag);
    v.add(&h);
    v.add(&g);
    sw.add(&v);
    w.add(&sw);

    // show main winow
    w.show_all();

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
fn create_db(path: &std::path::Path) -> sqlite::Connection {
    println!("Updating database..");

    let client = ureq::AgentBuilder::new()
        .user_agent("nanowave(https://github.com/sigmaSd/nanowave)")
        .build();
    let s: Vec<Station> = client
        .get("http://91.132.145.114/json/stations")
        .call()
        .unwrap()
        .into_json()
        .unwrap();

    let _ = std::fs::create_dir_all(&path);
    let _ = std::fs::remove_file(path.join("db.sqlite"));
    let _ = std::fs::remove_file(path.join("db.csv"));

    let mut o = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path.join("./db.csv"))
        .unwrap();
    writeln!(o, "Name,Country,Url").unwrap();
    for ss in s {
        writeln!(o, "{},{},{}", ss.name, ss.country, ss.url).unwrap();
    }

    let mut p = std::process::Command::new("sqlite3")
        .current_dir(path)
        .stdin(std::process::Stdio::piped())
        .arg("db.sqlite")
        .spawn()
        .unwrap();
    writeln!(
        p.stdin.as_mut().unwrap(),
        ".mode csv\n.import db.csv mytable"
    )
    .unwrap();
    p.wait().unwrap();

    println!("Done!");
    sqlite::open(path.join("db.sqlite")).unwrap()
}

#[derive(Debug, Deserialize)]
struct Station {
    name: String,
    url: String,
    country: String,
}
