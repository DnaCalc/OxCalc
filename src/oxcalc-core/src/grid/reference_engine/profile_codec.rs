//! OxFml binding-payload codec for the strict-excel-grid profile:
//! decoding profile payloads back into the grid reference AST and
//! building the profile reference records (atom, name, range, structured,
//! transformed) the bind profile emits. Internal to the reference engine;
//! shares the parent module's types and parse/render helpers via
//! `use super::*`.

use super::*;

#[must_use]
pub fn decode_excel_grid_reference_payload(payload: &ProfilePayload) -> Option<ExcelGridReference> {
    if payload.payload_kind != "excel-grid-reference" || payload.encoding != "json" {
        return None;
    }
    serde_json::from_str(&payload.data).ok()
}

#[must_use]
pub fn excel_grid_reference_like_from_profile_record(
    record: &ProfileReferenceRecord,
) -> Option<ReferenceLike> {
    if record.profile_id != EXCEL_GRID_PROFILE_ID {
        return None;
    }
    let reference = decode_excel_grid_reference_payload(&record.profile_payload)?;
    if record.normal_form_key != normal_form_key_for_reference(&record.profile_id, &reference) {
        return None;
    }
    let display_text = record
        .render_hint
        .clone()
        .unwrap_or_else(|| record.normal_form_key.0.clone());
    Some(ReferenceLike::opaque(
        ReferenceSystemId(EXCEL_GRID_PROFILE_ID.to_string()),
        ReferenceHandle {
            id: ReferenceHandleId::from_bytes(record.normal_form_key.0.clone().into_bytes()),
        },
        Some(ReferenceDisplay {
            text: ExcelText::from_interop_assignment(&display_text),
        }),
    ))
}

pub(super) fn profile_record_for_reference(
    profile_id: &str,
    request: &ReferenceAtomBindRequest,
    reference: ExcelGridReference,
    validity: ReferenceValidity,
) -> ProfileReferenceRecord {
    let normal_form_key = normal_form_key_for_reference(profile_id, &reference);
    let payload_data =
        serde_json::to_string(&reference).expect("excel grid reference payload serializes");
    ProfileReferenceRecord {
        profile_id: profile_id.to_string(),
        profile_version: ProfileVersion::v1(),
        source_info: ReferenceSourceInfo {
            source_channel: request.source_channel,
            source_span: request.source_span,
            source_text: request.source_text.clone(),
            parsed_qualifier: request.parsed_qualifier.clone(),
            address_fidelity: Some(request.source_text.clone()),
        },
        profile_payload: ProfilePayload {
            payload_kind: "excel-grid-reference".to_string(),
            encoding: "json".to_string(),
            data: payload_data,
        },
        normal_form_key,
        render_hint: Some(request.source_text.clone()),
        validity,
    }
}

pub(super) fn profile_record_for_name_reference(
    profile_id: &str,
    request: &ReferenceNameBindRequest,
    reference: ExcelGridReference,
    validity: ReferenceValidity,
) -> ProfileReferenceRecord {
    let normal_form_key = normal_form_key_for_reference(profile_id, &reference);
    let payload_data =
        serde_json::to_string(&reference).expect("excel grid reference payload serializes");
    ProfileReferenceRecord {
        profile_id: profile_id.to_string(),
        profile_version: ProfileVersion::v1(),
        source_info: ReferenceSourceInfo {
            source_channel: request.source_channel,
            source_span: request.source_span,
            source_text: request.source_text.clone(),
            parsed_qualifier: request.parsed_qualifier.clone(),
            address_fidelity: Some(request.source_text.clone()),
        },
        profile_payload: ProfilePayload {
            payload_kind: "excel-grid-reference".to_string(),
            encoding: "json".to_string(),
            data: payload_data,
        },
        normal_form_key,
        render_hint: Some(request.source_text.clone()),
        validity,
    }
}

pub(super) fn profile_record_for_structured_reference(
    profile_id: &str,
    request: &ReferenceStructuredBindRequest,
    reference: ExcelGridReference,
    validity: ReferenceValidity,
) -> ProfileReferenceRecord {
    let normal_form_key = normal_form_key_for_reference(profile_id, &reference);
    let payload_data =
        serde_json::to_string(&reference).expect("excel grid reference payload serializes");
    ProfileReferenceRecord {
        profile_id: profile_id.to_string(),
        profile_version: ProfileVersion::v1(),
        source_info: ReferenceSourceInfo {
            source_channel: request.source_channel,
            source_span: request.source_span,
            source_text: request.source_text.clone(),
            parsed_qualifier: None,
            address_fidelity: Some(request.source_text.clone()),
        },
        profile_payload: ProfilePayload {
            payload_kind: "excel-grid-reference".to_string(),
            encoding: "json".to_string(),
            data: payload_data,
        },
        normal_form_key,
        render_hint: Some(request.source_text.clone()),
        validity,
    }
}

pub(super) fn profile_record_for_range_reference(
    profile_id: &str,
    request: &ReferenceRangeBindRequest,
    reference: ExcelGridReference,
    validity: ReferenceValidity,
) -> ProfileReferenceRecord {
    let normal_form_key = normal_form_key_for_reference(profile_id, &reference);
    let payload_data =
        serde_json::to_string(&reference).expect("excel grid reference payload serializes");
    ProfileReferenceRecord {
        profile_id: profile_id.to_string(),
        profile_version: ProfileVersion::v1(),
        source_info: ReferenceSourceInfo {
            source_channel: request.source_channel,
            source_span: request.source_span,
            source_text: request.source_text.clone(),
            parsed_qualifier: common_range_qualifier(request),
            address_fidelity: Some(request.source_text.clone()),
        },
        profile_payload: ProfilePayload {
            payload_kind: "excel-grid-reference".to_string(),
            encoding: "json".to_string(),
            data: payload_data,
        },
        normal_form_key,
        render_hint: Some(request.source_text.clone()),
        validity,
    }
}

pub(super) fn profile_record_for_transformed_reference(
    original: &ProfileReferenceRecord,
    reference: ExcelGridReference,
    validity: ReferenceValidity,
    anchor_after: Option<&ExcelGridFormulaAnchor>,
) -> ProfileReferenceRecord {
    let normal_form_key = normal_form_key_for_reference(&original.profile_id, &reference);
    let render_hint = render_reference_for_channel(
        &reference,
        original.source_info.source_channel,
        anchor_after,
    )
    .unwrap_or_else(|| normal_form_key.0.clone());
    let payload_data =
        serde_json::to_string(&reference).expect("excel grid reference payload serializes");
    let mut source_info = original.source_info.clone();
    source_info.source_text = render_hint.clone();
    source_info.address_fidelity = Some(render_hint.clone());
    source_info.parsed_qualifier = transformed_parsed_qualifier(&reference);
    ProfileReferenceRecord {
        profile_id: original.profile_id.clone(),
        profile_version: original.profile_version.clone(),
        source_info,
        profile_payload: ProfilePayload {
            payload_kind: "excel-grid-reference".to_string(),
            encoding: "json".to_string(),
            data: payload_data,
        },
        normal_form_key,
        render_hint: Some(render_hint),
        validity,
    }
}
