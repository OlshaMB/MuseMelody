use audiotags::Tag;
use rodio::{Decoder, OutputStream, Sink, Source};
use soloud::*;
use std::{
    env,
    fs::File,
    io::{BufReader, Cursor},
    ops::Deref,
    rc::Rc,
    sync::{mpsc::channel, Arc, Mutex},
    thread,
    time::Duration,
};

fn main() {
    let app = MainWindow::new().unwrap();
    let mut sl = Arc::new(Mutex::new(Soloud::default().unwrap()));
    let album_cover: slint::Image;
    let path = env::current_dir()
        .unwrap()
        .join("resources/Alan Walker - Dreamer.flac");
    let tag = Tag::new().read_from_path(path.clone()).unwrap();
    app.global::<Track>().set_hasCoverImage(false);
    app.global::<Track>().set_paused(true);
    app.global::<Track>()
        .set_trackTitle(path.as_path().file_stem().unwrap().to_str().unwrap().into());
    if tag.album_cover().is_some() {
        let image = image::io::Reader::new(Cursor::new(tag.album_cover().unwrap().data))
            .with_guessed_format()
            .unwrap()
            .decode()
            .unwrap()
            .into_rgb8();
        let pixel_buffer = slint::SharedPixelBuffer::<slint::Rgb8Pixel>::clone_from_slice(
            image.as_raw(),
            image.width(),
            image.height(),
        );
        album_cover = slint::Image::from_rgb8(pixel_buffer);
        app.global::<Track>().set_hasCoverImage(true);
        app.global::<Track>().set_coverImage(album_cover);
    }
    let mut wav = audio::Wav::default();
    wav.load(path).unwrap();
    let mut handle: Arc<Mutex<Option<Handle>>> = Arc::new(Mutex::new(None));
    app.global::<Track>().set_trackDuration(wav.length() as i32);
    app.global::<Track>().on_pause({
        let ui = app.as_weak();
        let sl = sl.clone();
        let handle = handle.clone();
        move || {
            if ui.upgrade().unwrap().global::<Track>().get_paused() {
                println!("{:?}", handle);
                if handle.lock().unwrap().is_none() {
                    *handle.lock().unwrap() = Some(sl.as_ref().lock().unwrap().play(&wav));
                } else {
                    sl.as_ref()
                        .lock()
                        .unwrap()
                        .set_pause(handle.lock().unwrap().unwrap(), false);
                }
            } else {
                if handle.lock().unwrap().is_some() {
                    sl.as_ref()
                        .lock()
                        .unwrap()
                        .set_pause(handle.lock().unwrap().unwrap(), true);
                }
            }
            ui.upgrade()
                .unwrap()
                .global::<Track>()
                .set_paused(!ui.upgrade().unwrap().global::<Track>().get_paused());
        }
    });
    app.global::<Track>().on_scroll({
        let ui = app.as_weak();
        let sl = sl.clone();
        let handle = handle.clone();
        move || {
            println!(
                "{:?}",
                ui.upgrade()
                    .unwrap()
                    .global::<Track>()
                    .get_trackCurrentTime()
            );
            if handle.lock().unwrap().is_none() {
                return;
            }

            sl.as_ref()
                .lock()
                .unwrap()
                .seek(
                    handle.lock().unwrap().unwrap(),
                    ui.upgrade()
                        .unwrap()
                        .global::<Track>()
                        .get_trackCurrentTime()
                        .try_into()
                        .unwrap(),
                )
                .unwrap();
        }
    });

    thread::spawn({
        let state = app.as_weak();
        let sl = sl.clone();
        let handle = handle.clone();
        // let (sx, rx) = channel();
        move || {
            loop {
                if handle.lock().unwrap().is_none() {
                    // ui.upgrade().unwrap().global::<Track>().set_trackCurrentTime(
                    //     0
                    // )
                    thread::sleep(Duration::from_secs(1));
                    continue;
                }
                thread::sleep(Duration::from_secs(1));
                // println!("Test1");
                slint::invoke_from_event_loop({
                    let state = state.clone();
                    let sl = sl.clone();
                    let handle = handle.clone();
                    move || {
                        state
                            .upgrade()
                            .unwrap()
                            .global::<Track>()
                            .set_trackCurrentTime(
                                sl.as_ref()
                                    .lock()
                                    .unwrap()
                                    .stream_position(handle.lock().unwrap().unwrap())
                                    .round() as i32,
                            )
                    }
                });
            }
        }
        // rx.into_iter()
    });
    app.run().unwrap();
}
slint::include_modules!();
