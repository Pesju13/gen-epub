use consio::input;
use gen_epub::{add_file, gen_content, gen_cover, gen_page, gen_toc};
use gen_epub::{EpubData, CONTAINER, MIME_TYPE, PAGE_STYLES, STYLESHEET};
use std::path::Path;
use std::{
    env::current_dir,
    fs::{read_dir, File},
    io::Write,
    path::PathBuf,
};
use zip::{write::FileOptions, ZipWriter};

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let path = if let Some(path) = args.get(1)  {
        path.to_owned()
    }else {
        input!(print "请输入目录：").unwrap()
    };

    let path = if path.eq(".") {
        current_dir()?
    } else {
        PathBuf::from(path)
    };
    run(path)
}

fn run(path: impl AsRef<Path>) -> std::io::Result<()> {
    let path = path.as_ref().to_path_buf();
    let mut dirs = Vec::new();
    let mut files = Vec::new();
    for dir in read_dir(&path)? {
        let tmp = dir.unwrap().path();
        if tmp.is_dir() {
            dirs.push(tmp)
        } else {
            files.push(tmp)
        }
    }
    if !dirs.is_empty() {
        dirs.sort();
        let input = input!(print "目录{} 存在多个文件夹，输入1合并: ", path.display()).unwrap();
        if input.eq("1") {
            write_dir(path, dirs)?
        } else {
            for dir in dirs {
                run(dir)?;
            }
        }
    } else {
        files.sort();
        write_one(path, files)?;
    }
    Ok(())
}

fn write_dir(path: PathBuf, dirs: Vec<PathBuf>) -> std::io::Result<()> {
    let title = path.file_name().unwrap().to_str().unwrap();
    let mut zip_path = path.clone();
    zip_path.set_extension("epub");
    let options: FileOptions<()> = FileOptions::default()
        .compression_method(zip::CompressionMethod::Stored)
        .unix_permissions(0o755);
    let f = File::create(zip_path)?;
    let mut zip_writer = ZipWriter::new(f);
    zip_writer.add_directory("META-INF", options)?;
    let mut cover_path = path.clone();
    cover_path.push("cover_image.jpg");
    let mut cover = String::new();
    if cover_path.exists() {
        cover = "cover_image.jpg".to_owned();
        zip_writer.start_file("cover_image.jpg", options)?;
        zip_writer.write_all(&std::fs::read(cover_path)?)?;
    }
    let mut prefixes = Vec::new();
    for dir in dirs {
        let dir_name = dir.file_name().unwrap().to_string_lossy().to_string();
        let mut prefix = Vec::new(); 
        for f in read_dir(&dir)? {
            let fpath = f?.path();
            if fpath.is_file() {
                let name = fpath.strip_prefix(&path).unwrap().display().to_string().replace('\\', "-");
                zip_writer.start_file(name.as_str(), options)?;
                zip_writer.write_all(&std::fs::read(&fpath)?)?;
                prefix.push(name)
            }
        }
        prefix.sort();
        if cover.is_empty() {
           cover = prefix.first().map(|s|s.into()).unwrap_or_default();
        }
        prefixes.push((dir_name, prefix));
    }
    prefixes.sort_by(|v1, v2| {
        v1.0.cmp(&v2.0)
    });
    let cover_option = gen_cover(&cover);
    let mut toc_params = Vec::new();
    let mut imgs = Vec::new();
    let mut page_names = Vec::new();
    for (name, arr) in &prefixes {
        let name_html = format!("{name}.html");
        let page = gen_page(name, &name_html, arr);
        add_file!(zip_writer, options, page);
        for v in arr{
            imgs.push(v.to_owned())
        }
        toc_params.push((name_html.clone(), name.to_owned()));
        page_names.push(name_html);
    }
    let content = gen_content(title, &imgs, &page_names, &cover);
    let toc = gen_toc(&toc_params);
    add_file!(
        zip_writer,
        options,
        cover_option,
        content,
        toc,
        CONTAINER,
        STYLESHEET,
        PAGE_STYLES,
        MIME_TYPE
    );
    zip_writer.finish()?;
    Ok(())
}

fn write_one(path: PathBuf, files: Vec<PathBuf>) -> std::io::Result<()> {
    let title = path.file_name().unwrap().to_str().unwrap();
    let mut zip_path = path.clone();
    zip_path.set_extension("epub");
    let options: FileOptions<()> = FileOptions::default()
        .compression_method(zip::CompressionMethod::Stored)
        .unix_permissions(0o755);
    let f = File::create(zip_path)?;
    let mut zip_writer = ZipWriter::new(f);
    zip_writer.add_directory("META-INF", options)?;
    let mut ps = Vec::new();
    let mut cover = String::new();
    for f in &files {
        let name = f.file_name().unwrap().to_str().unwrap();
        if name.starts_with("cover_image") {
            zip_writer.start_file(cover.as_str(), options)?;
            zip_writer.write_all(&std::fs::read(format!("{}/{cover}", path.display()))?)?;
            cover = name.to_owned();
        } else {
            ps.push(name.to_string());
            cover = if cover.is_empty() {
                files[0].file_name().unwrap().to_string_lossy().to_string()
            } else {
                cover
            };
            zip_writer.start_file(name, options)?;
            zip_writer.write_all(&std::fs::read(f)?)?;

        }
    }
    let cover_option = gen_epub::gen_cover(&cover);
    let page = gen_epub::gen_page("00", "index.html", &ps);
    let content = gen_epub::gen_content(title, &ps, &["index.html".to_owned()], &cover);
    let toc = gen_epub::gen_toc(&[("index.html".to_owned(), "正文".to_owned())]);
    add_file!(
        zip_writer,
        options,
        cover_option,
        page,
        content,
        toc,
        CONTAINER,
        STYLESHEET,
        PAGE_STYLES,
        MIME_TYPE
    );
    zip_writer.finish()?;
    Ok(())
}
