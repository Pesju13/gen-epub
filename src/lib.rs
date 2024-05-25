
pub trait EpubData {
    fn name(&self) -> &str;
    fn data(&self) -> &[u8];
}

#[derive(Clone)]
pub struct ConstEpubOption {
    pub name: &'static str,
    pub content: &'static str,
}
impl EpubData for ConstEpubOption {
    fn name(&self) -> &str {
        self.name
    }

    fn data(&self) -> &[u8] {
        self.content.as_bytes()
    }
}
pub fn get_const_contents() -> Vec<ConstEpubOption> {
    vec![
        MIME_TYPE.clone(),
        PAGE_STYLES.clone(),
        STYLESHEET.clone(),
        CONTAINER.clone(),
    ]
}
pub static MIME_TYPE: ConstEpubOption = ConstEpubOption {
    name: "mimetype",
    content: "application/epub+zip",
};

pub static PAGE_STYLES: ConstEpubOption = ConstEpubOption {
    name: "page_styles.css",
    content: include_str!("../page_styles.css"),
};
pub static STYLESHEET: ConstEpubOption = ConstEpubOption {
    name: "stylesheet.css",
    content: include_str!("../stylesheet.css"),
};

pub static CONTAINER: ConstEpubOption = ConstEpubOption {
    name: "META-INF/container.xml",
    content: include_str!("../META-INF/container.xml"),
};

pub static CONTENT_STR: &str = include_str!("../content.opf");
pub static PAGE: &str = include_str!("../index.html");
pub static TITLE_PAGE: &str = include_str!("../titlepage.xhtml");
pub static TOC: &str = include_str!("../toc.ncx");

fn p_from(src: &str) -> String {
    r#"<p class="calibre1"><a id="p3"></a><img src="{$src}" class="calibre2" /></p>"#
        .replace("{$src}", src)
}
#[macro_export]
macro_rules! add_file {
    ($zip:expr, $op:expr, $($data:expr), +) => {
        {
            $(
                $zip.start_file($data.name(), $op)?;
                $zip.write_all($data.data())?;       
            )+
        }
    };
}

pub struct EpubOption {
    pub name: String,
    pub content: String,
}
impl EpubData for EpubOption {
    fn name(&self) -> &str {
        &self.name
    }

    fn data(&self) -> &[u8] {
        self.content.as_bytes()
    }
}

fn item_img_from(src: &str) -> String {
    let id = BASE64_URL_SAFE.encode(format!("{src}{}", rand::random::<u32>()));
    r#"    <item href="{$href}" id="{$id}" media-type="image/jpeg"/>"#
        .replace("{$id}", &id)
        .replace("{$href}", src)
}

fn item_html_from(src: &str) -> (String, String) {
    let id = BASE64_URL_SAFE.encode(format!("{src}{}", rand::random::<u32>()));
    (
        r#"    <item href="{$href}" id="{$id}" media-type="application/xhtml+xml"/>"#
            .replace("{$id}", &id)
            .replace("{$href}", src),
        id,
    )
}

pub fn gen_page(title: &str, name: &str, ps: &[String]) -> EpubOption {
    let mut body = String::new();
    for p in ps {
        body.push_str(&p_from(p));
    }
    let page = PAGE.replace("{$title}", title).replace("{$body}", &body);
    EpubOption {
        name: name.to_owned(),
        content: page,
    }
}
pub fn gen_content(title: &str, imgs: &[String], htmls: &[String], cover: &str) -> EpubOption {
    let mut item_imgs = String::new();
    let mut item_htmls = String::new();
    let mut itemrefs = String::new();
    for img in imgs {
        item_imgs.push_str(&item_img_from(img))
    }
    for html in htmls {
        let (html, id) = item_html_from(html);
        item_htmls.push_str(&html);
        itemrefs.push_str(&r#"    <itemref idref="{$id}"/>"#.replace("{$id}", &id));
    }

    let content = CONTENT_STR
        .replace("{$title}", title)
        .replace("{$cover}", cover)
        .replace("{$item_img}", &item_imgs)
        .replace("{$item_html}", &item_htmls)
        .replace("{$itemref}", &itemrefs);

    EpubOption {
        name: "content.opf".to_owned(),
        content,
    }
}

use base64::{prelude::BASE64_URL_SAFE, Engine};
fn nav_from(src: &str, text: &str) -> String {
    let point = r#"    <navPoint id="{$id}" playOrder="1">
      <navLabel>
        <text>{$text}</text>
      </navLabel>
      <content src="{$src}"/>
    </navPoint>"#;
    let id = BASE64_URL_SAFE.encode(format!("{src}-{text}{}", rand::random::<u64>()));
    point
        .replace("{$id}", &id)
        .replace("{$src}", src)
        .replace("{$text}", text)
}

pub fn gen_toc(navs: &[(String, String)]) -> EpubOption {
    let mut nav_points = String::new();
    for (nav, text) in navs {
        nav_points.push_str(&nav_from(nav, text))
    }
    let toc = TOC.replace("{$navPoint}", &nav_points);
    // std::fs::write(format!("{path}/toc.ncx"), toc)?;
    EpubOption {
        name: "toc.ncx".to_owned(),
        content: toc,
    }
}

pub fn gen_cover(cover: &str) -> EpubOption {
    let xhtml = TITLE_PAGE.replace("{$cover}", cover);
    EpubOption {
        name: "titlepage.xhtml".to_owned(),
        content: xhtml,
    }
}
