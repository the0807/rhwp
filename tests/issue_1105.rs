//! Issue #1105: HWP3-origin HWP5 conversion keeps Hancom page break around sample16 p21.

use std::fs;
use std::path::Path;

fn load_doc(rel_path: &str) -> rhwp::wasm_api::HwpDocument {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(rel_path);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {rel_path}: {e}"));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {rel_path}: {e:?}"))
}

#[test]
fn task1105_sample16_hwp5_page_break_before_section_4_matches_hancom() {
    let doc = load_doc("samples/hwp3-sample16-hwp5.hwp");
    assert_eq!(doc.page_count(), 64);

    let page20 = doc.dump_page_items(Some(19));
    assert!(page20.contains("Table          pi=425"));
    assert!(
        !page20.contains("pi=426"),
        "IDC center heading must start after the visible page break:\n{page20}"
    );

    let page21 = doc.dump_page_items(Some(20));
    assert!(page21.contains("FullParagraph  pi=426"));
    assert!(page21.contains("FullParagraph  pi=427"));
    assert!(page21.contains("FullParagraph  pi=439"));
    assert!(
        !page21.contains("pi=440"),
        "section 4 heading must not remain at the end of page 21:\n{page21}"
    );

    let page22 = doc.dump_page_items(Some(21));
    assert!(page22.contains("FullParagraph  pi=440"));
    assert!(page22.contains("Table          pi=441"));
    assert!(page22.contains("FullParagraph  pi=449"));
    assert!(
        !page22.contains("pi=450"),
        "firewall paragraph must not leak into page 22:\n{page22}"
    );

    let page23 = doc.dump_page_items(Some(22));
    assert!(
        page23.contains("FullParagraph  pi=450"),
        "firewall paragraph must start page 23:\n{page23}"
    );
    assert!(
        page23.contains("pi=460"),
        "page 23 must reach the integrated DB cluster paragraph:\n{page23}"
    );
    assert!(
        !page23.contains("pi=461"),
        "next target-system paragraph must not remain on page 23:\n{page23}"
    );
}

fn assert_sample16_hwp5_business_selection_starts_next_page(rel_path: &str) {
    let doc = load_doc(rel_path);
    assert_eq!(doc.page_count(), 64, "{rel_path}");

    let page4 = doc.dump_page_items(Some(3));
    assert!(
        page4.contains("FullParagraph  pi=118"),
        "WAN line must stay on page 4 for {rel_path}:\n{page4}"
    );

    let page5 = doc.dump_page_items(Some(4));
    assert!(
        !page5.contains("pi=118"),
        "page 5 must start at the information-system importance paragraph, not the WAN line, for {rel_path}:\n{page5}"
    );
    assert!(
        page5.contains("FullParagraph  pi=119"),
        "information-system importance paragraph must start page 5 for {rel_path}:\n{page5}"
    );
    assert!(
        page5.contains("FullParagraph  pi=140"),
        "joint supply paragraph must close page 5 for {rel_path}:\n{page5}"
    );
    assert!(
        !page5.contains("pi=141"),
        "business selection heading must not remain on page 5 for {rel_path}:\n{page5}"
    );

    let page6 = doc.dump_page_items(Some(5));
    assert!(
        page6.contains("FullParagraph  pi=141"),
        "business selection heading must start page 6 for {rel_path}:\n{page6}"
    );
    assert!(
        page6.contains("FullParagraph  pi=142"),
        "business selection body must follow the heading for {rel_path}:\n{page6}"
    );
    assert!(
        page6.contains("FullParagraph  pi=144"),
        "successful bidder heading must remain on page 6 for {rel_path}:\n{page6}"
    );
}

fn assert_sample16_hwp5_server_requirements_page_matches_hancom(rel_path: &str) {
    let doc = load_doc(rel_path);
    assert_eq!(doc.page_count(), 64, "{rel_path}");

    let page22 = doc.dump_page_items(Some(21));
    assert!(
        page22.contains("FullParagraph  pi=449"),
        "RDBMS paragraph must close page 22 for {rel_path}:\n{page22}"
    );
    assert!(
        !page22.contains("pi=450"),
        "firewall paragraph must not leak into page 22 for {rel_path}:\n{page22}"
    );

    let page23 = doc.dump_page_items(Some(22));
    assert!(
        page23.contains("FullParagraph  pi=450"),
        "firewall paragraph must start page 23 for {rel_path}:\n{page23}"
    );
    assert!(
        page23.contains("FullParagraph  pi=451"),
        "hardware/software paragraph must follow on page 23 for {rel_path}:\n{page23}"
    );
    assert!(
        page23.contains("pi=460"),
        "page 23 must reach the integrated DB cluster paragraph for {rel_path}:\n{page23}"
    );
    assert!(
        !page23.contains("pi=461"),
        "next target-system paragraph must not remain on page 23 for {rel_path}:\n{page23}"
    );
}

#[test]
fn task1105_sample16_hwp5_2010_business_selection_break_matches_hancom() {
    assert_sample16_hwp5_business_selection_starts_next_page("samples/hwp3-sample16-hwp5-2010.hwp");
}

#[test]
fn task1105_sample16_hwp5_2010_server_requirements_page_matches_hancom() {
    assert_sample16_hwp5_server_requirements_page_matches_hancom(
        "samples/hwp3-sample16-hwp5-2010.hwp",
    );
}

#[test]
fn task1105_sample16_hwp5_2018_business_selection_break_matches_hancom() {
    assert_sample16_hwp5_business_selection_starts_next_page("samples/hwp3-sample16-hwp5-2018.hwp");
}

#[test]
fn task1105_sample16_hwp5_2018_server_requirements_page_matches_hancom() {
    assert_sample16_hwp5_server_requirements_page_matches_hancom(
        "samples/hwp3-sample16-hwp5-2018.hwp",
    );
}

#[test]
fn task1105_sample16_hwp5_2022_business_selection_break_matches_hancom() {
    assert_sample16_hwp5_business_selection_starts_next_page("samples/hwp3-sample16-hwp5-2022.hwp");
}

#[test]
fn task1105_sample16_hwp5_2022_server_requirements_page_matches_hancom() {
    assert_sample16_hwp5_server_requirements_page_matches_hancom(
        "samples/hwp3-sample16-hwp5-2022.hwp",
    );
}

#[test]
fn task1105_sample16_hwp5_2024_business_selection_break_matches_hancom() {
    assert_sample16_hwp5_business_selection_starts_next_page("samples/hwp3-sample16-hwp5-2024.hwp");
}

#[test]
fn task1105_sample16_hwp5_2024_server_requirements_page_matches_hancom() {
    assert_sample16_hwp5_server_requirements_page_matches_hancom(
        "samples/hwp3-sample16-hwp5-2024.hwp",
    );
}

#[test]
fn task1105_k_water_rfp_2024_page_count_matches_hancom_pdf() {
    let doc = load_doc("samples/k-water-rfp-2024.hwp");
    assert_eq!(doc.page_count(), 27);
}

#[test]
fn task1105_k_water_rfp_2024_first_rowspan_table_keeps_line_reset_split() {
    let doc = load_doc("samples/k-water-rfp-2024.hwp");

    let page5 = doc.dump_page_items(Some(4));
    assert!(
        page5.contains("PartialTable   pi=52 ci=0  rows=0..4"),
        "the first large rowspan table must split inside its last row:\n{page5}"
    );
    assert!(
        page5.contains("end_cut=["),
        "the first large rowspan table must carry row-block line cuts:\n{page5}"
    );
}

#[test]
fn task1105_k_water_rfp_2024_cover_hides_first_page_footer() {
    let doc = load_doc("samples/k-water-rfp-2024.hwp");
    let svg = doc
        .render_page_svg_native(0)
        .expect("render k-water-rfp-2024 page 1");

    assert!(
        !svg.contains(
            r##"<line x1="80" y1="1034.8666666666668" x2="713.96" y2="1034.8666666666668" stroke="#787878" stroke-width="1.5"/>"##
        ),
        "first page footer table line must be hidden by SectionDef first-page footer hide"
    );
}
