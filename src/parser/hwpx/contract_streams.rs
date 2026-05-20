//! [Task #852] HWPX → HWP OLE contract 스트림 변환
//!
//! 한컴 HWP 5.0 정답지는 다음 9 스트림 contract 를 요구한다:
//!
//! ```text
//! FileHeader, DocInfo, BodyText/Section0,        // 본문 (rhwp 기본 작성)
//! HwpSummaryInformation, DocOptions/_LinkDoc,    // 메타 (본 모듈)
//! Scripts/DefaultJScript, Scripts/JScriptVersion, // 스크립트 (본 모듈)
//! PrvImage, PrvText                              // 미리보기 (본 모듈)
//! ```
//!
//! HWPX 컨테이너에 동등 데이터가 있으면 (Preview, Scripts) 변환·passthrough,
//! 없으면 (HwpSummary, DocOptions/_LinkDoc, Scripts/JScriptVersion) 정적
//! fallback (`saved/blank2010.hwp` 추출) 사용.
//!
//! Stage 2.1 = HWPX 컨테이너 → extra_streams (정공법, Preview/Scripts).
//! Stage 2.2 (본 모듈) = 정적 fallback (HwpSummary/DocOptions/JScriptVersion).

use super::reader::HwpxReader;

// [Task #852 Stage 2.2] 정적 fallback 자산.
//
// HWPX 컨테이너에 동등 데이터가 없는 contract 스트림 3 개를 `samples/form-01.hwp`
// (한컴 정답지) 및 `saved/blank2010.hwp` 에서 사전 추출. 변환 결과가 한컴
// 호환을 보장하기 위한 최소 contract.
//
// - hwp_summary_information.bin (461 B) — `samples/form-01.hwp` 추출.
//   title/creator/subject 등 OLE Property Set. HWPX content.hpf opf:metadata
//   기반 패치는 후속 task.
// - doc_options_link_doc.bin (524 B) — `saved/blank2010.hwp` 추출. UTF-16 LE
//   임시 파일 경로 메타.
// - scripts_jscript_version.bin (13 B) — `saved/blank2010.hwp` 추출.
//   `cd 64 80 00 ...` 13 바이트 헤더.
const FALLBACK_HWP_SUMMARY: &[u8] = include_bytes!("blank2010_assets/hwp_summary_information.bin");
const FALLBACK_DOC_OPTIONS_LINK_DOC: &[u8] =
    include_bytes!("blank2010_assets/doc_options_link_doc.bin");
const FALLBACK_SCRIPTS_JSCRIPT_VERSION: &[u8] =
    include_bytes!("blank2010_assets/scripts_jscript_version.bin");

/// HWPX 컨테이너 → HWP OLE 스트림 매핑 결과
pub(super) struct ContractStreams {
    /// `Vec<(path, data)>` 형태 — Document::extra_streams 에 그대로 주입 가능
    pub streams: Vec<(String, Vec<u8>)>,
}

/// HWPX ZIP reader 로부터 contract 스트림 4 개를 추출/변환.
///
/// - `Preview/PrvText.txt` (UTF-8) → `/PrvText` (UTF-16 LE)
/// - `Preview/PrvImage.png` → `/PrvImage` (passthrough)
/// - `Scripts/sourceScripts` → `/Scripts/DefaultJScript` (zlib deflate)
/// - `Scripts/headerScripts` → 정적 fallback 사용 (Stage 2.2)
///
/// HWPX 에 동등 파일이 없으면 해당 스트림은 생략. cfb_writer 가 한컴 정답지
/// 와 비교하여 추가 fallback (HwpSummary / DocOptions/_LinkDoc) 가 필요.
pub(super) fn extract_contract_streams(reader: &mut HwpxReader) -> ContractStreams {
    let mut streams = Vec::new();

    // PrvText.txt (UTF-8) → /PrvText (UTF-16 LE, HWP5 spec)
    if let Ok(prv_text_utf8) = reader.read_file("Preview/PrvText.txt") {
        let utf16_bytes: Vec<u8> = prv_text_utf8
            .encode_utf16()
            .flat_map(|c| c.to_le_bytes())
            .collect();
        streams.push(("/PrvText".to_string(), utf16_bytes));
    }

    // PrvImage.png → /PrvImage (PNG passthrough)
    if let Ok(prv_image_bytes) = reader.read_file_bytes("Preview/PrvImage.png") {
        streams.push(("/PrvImage".to_string(), prv_image_bytes));
    }

    // Scripts/sourceScripts → /Scripts/DefaultJScript (zlib deflate)
    if let Ok(source_scripts_bytes) = reader.read_file_bytes("Scripts/sourceScripts") {
        if let Some(compressed) = zlib_deflate(&source_scripts_bytes) {
            streams.push(("/Scripts/DefaultJScript".to_string(), compressed));
        }
    }

    // [Stage 2.2] HWPX 컨테이너에 동등 데이터가 없는 contract 3 스트림 fallback.
    // 한컴 정답지가 모든 HWP 파일에 요구하는 최소 OLE Property Set / 메타.
    //
    // 주의: 스트림 경로는 OLE 표준의 `\x05` 선두 prefix (Property Set 표시) 가
    // 정답지의 실제 이름이나, mini_cfb 가 path 형식으로 처리하므로 일반
    // ASCII path 로 작성 후 한컴이 정상 인식. form-01 정답지의 실제 경로
    // `\x05HwpSummaryInformation` 와 byte-level 차이는 Stage 2.3 검증.
    streams.push((
        "/HwpSummaryInformation".to_string(),
        FALLBACK_HWP_SUMMARY.to_vec(),
    ));
    streams.push((
        "/DocOptions/_LinkDoc".to_string(),
        FALLBACK_DOC_OPTIONS_LINK_DOC.to_vec(),
    ));
    streams.push((
        "/Scripts/JScriptVersion".to_string(),
        FALLBACK_SCRIPTS_JSCRIPT_VERSION.to_vec(),
    ));

    ContractStreams { streams }
}

/// 단순 zlib deflate 헬퍼. 실패 시 None.
fn zlib_deflate(input: &[u8]) -> Option<Vec<u8>> {
    use flate2::write::ZlibEncoder;
    use flate2::Compression;
    use std::io::Write;

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(input).ok()?;
    encoder.finish().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zlib_deflate_roundtrip() {
        use flate2::read::ZlibDecoder;
        use std::io::Read;

        let input = b"hello rhwp scripts test";
        let compressed = zlib_deflate(input).expect("zlib deflate failed");
        let mut decoder = ZlibDecoder::new(&compressed[..]);
        let mut decoded = Vec::new();
        decoder
            .read_to_end(&mut decoded)
            .expect("zlib inflate failed");
        assert_eq!(decoded, input);
    }
}
