use std::fs;
use std::fs::File;
use std::io::Read;

use serde_json::json;
use sha2::{Digest, Sha256};
use tabula_cli::commands::{rebuild_images, strip_images};
use tabula_cli_tests::tabula_cli_test_utils;
use tempfile::TempDir;
use zip::ZipArchive;

#[test]
fn rebuild_images_restores_from_cache() {
    let temp_dir = TempDir::new().expect("temp dir");
    let git_dir = temp_dir.path().join(".git");
    fs::create_dir_all(&git_dir).expect("git dir");

    let xlsm_path = temp_dir.path().join("input.xlsm");
    let (order, img1, img2) =
        tabula_cli_test_utils::create_xlsm_with_images(&xlsm_path).expect("xlsm");

    let stripped = temp_dir.path().join("stripped.xlsm");
    strip_images::strip_images(Some(xlsm_path.clone()), Some(stripped.clone())).expect("strip");

    rebuild_images::rebuild::rebuild_images(Some(stripped.clone()), false, false).expect("rebuild");

    let mut archive = ZipArchive::new(File::open(&stripped).expect("open output")).expect("zip");
    let mut names = Vec::new();
    for i in 0..archive.len() {
        names.push(archive.by_index(i).expect("entry").name().to_string());
    }
    assert_eq!(names, order);

    let mut img1_data = Vec::new();
    archive
        .by_name("xl/media/image1.jpg")
        .expect("image1")
        .read_to_end(&mut img1_data)
        .expect("read");
    assert_eq!(img1_data, img1);
    let mut img2_data = Vec::new();
    archive
        .by_name("xl/media/image2.png")
        .expect("image2")
        .read_to_end(&mut img2_data)
        .expect("read");
    assert_eq!(img2_data, img2);
}

#[test]
fn rebuild_images_from_urls_with_stubbed_downloader() {
    let temp_dir = TempDir::new().expect("temp dir");
    let xlsm_path = temp_dir.path().join("url.xlsm");
    build_minimal_image_workbook(&xlsm_path);

    let downloaded = b"new-bytes".to_vec();
    let expected = downloaded.clone();
    let downloader = move |urls: &std::collections::BTreeMap<usize, String>,
                           _warnings: &mut Vec<String>|
          -> anyhow::Result<std::collections::BTreeMap<usize, Vec<u8>>> {
        assert_eq!(urls.len(), 1);
        assert_eq!(urls.get(&0).unwrap(), "https://example.invalid/image1.jpeg");
        let mut map = std::collections::BTreeMap::new();
        map.insert(0, downloaded.clone());
        Ok(map)
    };

    rebuild_images::rebuild::rebuild_images_from_urls_with_downloader(
        Some(xlsm_path.clone()),
        &downloader,
    )
    .expect("rebuild");

    let mut archive = ZipArchive::new(File::open(&xlsm_path).expect("open output")).expect("zip");
    let mut img_data = Vec::new();
    archive
        .by_name("xl/media/image1.jpeg")
        .expect("image1")
        .read_to_end(&mut img_data)
        .expect("read");
    assert_eq!(img_data, expected);

    let mut rels_data = Vec::new();
    archive
        .by_name("xl/richData/_rels/rdRichValueWebImage.xml.rels")
        .expect("rels")
        .read_to_end(&mut rels_data)
        .expect("read rels");
    assert!(
        std::str::from_utf8(&rels_data).unwrap().contains("https://example.invalid/image1.jpeg")
    );
}

#[test]
fn rebuild_images_handles_missing_optional_entries() {
    let temp_dir = TempDir::new().expect("temp dir");
    let git_dir = temp_dir.path().join(".git");
    fs::create_dir_all(&git_dir).expect("git dir");
    let cache_dir = git_dir.join("xlsm_image_cache");
    fs::create_dir_all(&cache_dir).expect("cache dir");

    let xlsm_path = temp_dir.path().join("input.xlsm");
    let (order, img1, img2) =
        tabula_cli_test_utils::create_xlsm_with_images(&xlsm_path).expect("xlsm");

    let hash1 = Sha256::digest(&img1).iter().map(|b| format!("{b:02x}")).collect::<String>();
    let hash2 = Sha256::digest(&img2).iter().map(|b| format!("{b:02x}")).collect::<String>();
    fs::write(cache_dir.join(&hash1), &img1).expect("cache1");
    fs::write(cache_dir.join(&hash2), &img2).expect("cache2");

    let mut file_order = order.clone();
    file_order.push("xl/calcChain.xml".to_string());
    let manifest = json!({
        "version": 1,
        "file_order": file_order,
        "images": {
            "xl/media/image1.jpg": {"hash": hash1, "size": img1.len()},
            "xl/media/image2.png": {"hash": hash2, "size": img2.len()}
        }
    });
    fs::write(cache_dir.join("_xlsm_manifest.json"), manifest.to_string()).expect("manifest");

    rebuild_images::rebuild::rebuild_images(Some(xlsm_path.clone()), false, false)
        .expect("rebuild");

    let mut archive = ZipArchive::new(File::open(&xlsm_path).expect("open output")).expect("zip");
    let mut img1_data = Vec::new();
    archive
        .by_name("xl/media/image1.jpg")
        .expect("image1")
        .read_to_end(&mut img1_data)
        .expect("read");
    assert_eq!(img1_data, img1);
    let mut img2_data = Vec::new();
    archive
        .by_name("xl/media/image2.png")
        .expect("image2")
        .read_to_end(&mut img2_data)
        .expect("read");
    assert_eq!(img2_data, img2);
}

fn build_minimal_image_workbook(path: &std::path::Path) {
    use std::io::Write;

    use zip::write::FileOptions;
    let file = File::create(path).expect("create");
    let mut writer = zip::ZipWriter::new(file);
    let time = zip::DateTime::from_date_and_time(1980, 1, 1, 0, 0, 0).expect("valid zip time");
    let deflated = FileOptions::<()>::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .last_modified_time(time);
    let stored = FileOptions::<()>::default()
        .compression_method(zip::CompressionMethod::Stored)
        .last_modified_time(time);

    writer.start_file("[Content_Types].xml", deflated).expect("types");
    writer
        .write_all(
            br#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
<Default Extension="xml" ContentType="application/xml"/>
<Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>
<Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>
<Override PartName="/xl/metadata.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheetMetadata+xml"/>
<Override PartName="/xl/richData/rdrichvalue.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.richData+xml"/>
<Override PartName="/xl/richData/rdRichValueWebImage.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.rdRichValueWebImage+xml"/>
<Override PartName="/xl/richData/_rels/rdRichValueWebImage.xml.rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
<Override PartName="/xl/_rels/workbook.xml.rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
<Override PartName="/_rels/.rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
<Override PartName="/xl/media/image1.jpeg" ContentType="image/jpeg"/>
</Types>"#,
        )
        .expect("write types");

    writer.start_file("_rels/.rels", deflated).expect("rels");
    writer
        .write_all(
            br#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>
</Relationships>"#,
        )
        .expect("write rels");

    writer.start_file("xl/_rels/workbook.xml.rels", deflated).expect("wb rels");
    writer
        .write_all(
            br#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>
<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/sheetMetadata" Target="metadata.xml"/>
<Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/richData" Target="richData/rdrichvalue.xml"/>
<Relationship Id="rId4" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/richData" Target="richData/rdRichValueWebImage.xml"/>
</Relationships>"#,
        )
        .expect("write wb rels");

    writer.start_file("xl/workbook.xml", deflated).expect("workbook");
    writer
        .write_all(
            br#"<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
<sheets>
<sheet name="Cards" sheetId="1" r:id="rId1"/>
</sheets>
</workbook>"#,
        )
        .expect("write workbook");

    writer.start_file("xl/metadata.xml", deflated).expect("metadata");
    writer
        .write_all(
            br#"<metadata xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:rd="http://schemas.microsoft.com/office/spreadsheetml/2017/richdata">
<futureMetadata count="1">
<bk>
<rd:rvb i="0"/>
</bk>
</futureMetadata>
<valueMetadata count="1">
<bk>
<rc t="1" v="0"/>
</bk>
</valueMetadata>
</metadata>"#,
        )
        .expect("write metadata");

    writer.start_file("xl/richData/rdrichvalue.xml", deflated).expect("rv");
    writer
        .write_all(
            br#"<rdrichvalue xmlns="http://schemas.microsoft.com/office/spreadsheetml/2017/richdata">
<rv><v>0</v></rv>
</rdrichvalue>"#,
        )
        .expect("write rv");

    writer.start_file("xl/richData/rdRichValueWebImage.xml", deflated).expect("webimage");
    writer
        .write_all(
            br#"<rdRichValueWebImage xmlns="http://schemas.microsoft.com/office/spreadsheetml/2020/richdatawebimage" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
<webImageSrds>
<webImageSrd>
<address r:id="rIdAddr1"/>
<blip r:id="rIdImg1"/>
</webImageSrd>
</webImageSrds>
</rdRichValueWebImage>"#,
        )
        .expect("write web image");

    writer
        .start_file("xl/richData/_rels/rdRichValueWebImage.xml.rels", deflated)
        .expect("web rels");
    writer
        .write_all(
            br#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rIdAddr1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink" Target="https://example.invalid/original" TargetMode="External"/>
<Relationship Id="rIdImg1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="../media/image1.jpeg"/>
</Relationships>"#,
        )
        .expect("write web rels");

    writer.start_file("xl/worksheets/sheet1.xml", deflated).expect("sheet1");
    writer
        .write_all(
            br#"<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
<sheetData>
<row r="1">
<c r="A1" vm="1">
<f>IMAGE("https://example.invalid/image1.jpeg")</f>
</c>
</row>
</sheetData>
</worksheet>"#,
        )
        .expect("write sheet");

    writer.start_file("xl/media/image1.jpeg", stored).expect("image");
    writer.write_all(b"original-bytes").expect("write image");

    writer.finish().expect("finish zip");
}
