use kasa_ai::{BoundingBox, Point, TextRegion};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type, ToSchema, uniffi::Record)]
#[serde(rename_all = "camelCase")]
pub struct OcrPoint {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type, ToSchema, uniffi::Record)]
#[serde(rename_all = "camelCase")]
pub struct OcrBoundingBox {
    pub points: Vec<OcrPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type, ToSchema, uniffi::Record)]
#[serde(rename_all = "camelCase")]
pub struct OcrTextRegion {
    pub bounding_box: OcrBoundingBox,
    pub confidence: Option<f32>,
    pub text: Option<String>,
}

impl From<Point> for OcrPoint {
    fn from(point: Point) -> Self {
        Self {
            x: point.x,
            y: point.y,
        }
    }
}

impl From<BoundingBox> for OcrBoundingBox {
    fn from(bounding_box: BoundingBox) -> Self {
        Self {
            points: bounding_box.points.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<TextRegion> for OcrTextRegion {
    fn from(region: TextRegion) -> Self {
        Self {
            bounding_box: region.bounding_box.into(),
            confidence: region.confidence,
            text: region.text.map(|text| text.to_string()),
        }
    }
}
