/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[derive(Debug, Clone, Copy)]
#[allow(non_snake_case)]
pub struct BorderInstance {
    pub aTaskOrigin: [f32; 2],
    pub aRect: [f32; 4],
    pub aColor0: [f32; 4],
    pub aColor1: [f32; 4],
    pub aFlags: i32,
    pub aWidths: [f32; 2],
    pub aRadii: [f32; 2],
    pub aClipParams1: [f32; 4],
    pub aClipParams2: [f32; 4],
}

#[derive(Debug, Clone, Copy)]
#[allow(non_snake_case)]
pub struct GradientInstance {
    pub aTaskRect: [f32 ;4],
    pub aAxisSelect: f32,
    pub aStops: [f32 ;4],
    pub aColor0: [f32 ;4],
    pub aColor1: [f32 ;4],
    pub aColor2: [f32 ;4],
    pub aColor3: [f32 ;4],
    pub aStartStop: [f32 ;2],
}

#[derive(Debug, Clone, Copy)]
#[allow(non_snake_case)]
pub struct SvgFilterInstance {
    pub aData: [i32; 4],
    pub aFilterRenderTaskAddress: i32,
    pub aFilterInput1TaskAddress: i32,
    pub aFilterInput2TaskAddress: i32,
    pub aFilterKind: i32,
    pub aFilterInputCount: i32,
    pub aFilterGenericInt: i32,
    pub aFilterExtraDataAddress: [i32; 2],
}

#[derive(Debug, Clone, Copy)]
#[allow(non_snake_case)]
pub struct ScalingInstance {
    pub aData: [i32; 4],
    pub aScaleTargetRect: [f32; 4],
    pub aScaleSourceRect: [i32; 4],
    pub aScaleSourceLayer: i32,
}

#[derive(Debug, Clone, Copy)]
#[allow(non_snake_case)]
pub struct CompositeInstance {
    aDeviceRect: [f32; 4],
    aDeviceClipRect: [f32; 4],
    aColor: [f32; 4],
    aLayer: f32,
    aZId: f32,
}

#[derive(Debug, Clone, Copy)]
#[allow(non_snake_case)]
pub struct PlsInstance {
    aRect: [f32; 4],
}

#[derive(Debug, Clone, Copy)]
#[allow(non_snake_case)]
pub struct BlurInstance {
    pub aData: [i32; 4],
    pub aBlurRenderTaskAddress: i32,
    pub aBlurSourceTaskAddress: i32,
    pub aBlurDirection: i32,
}

#[derive(Debug, Clone, Copy)]
#[allow(non_snake_case)]
pub struct ClipMaskInstance {
    aTransformIds: [i32; 2],
    aClipDataResourceAddress: [i32; 4],
    aClipLocalPos: [f32; 2],
    aClipTileRect: [f32; 4],
    aClipDeviceArea: [f32; 4],
    aClipOrigins: [f32; 4],
    aDevicePixelScale: f32,
}

#[derive(Debug, Clone, Copy)]
#[allow(non_snake_case)]
pub struct DebugColorVertex {
    pub aPosition: [f32; 3],
    pub aColor: [f32; 4],
}

#[derive(Debug, Clone, Copy)]
#[allow(non_snake_case)]
pub struct DebugFontVertex {
    pub aPosition: [f32; 3],
    pub aColor: [f32; 4],
    pub aColorTexCoord: [f32; 2],
}

#[derive(Debug, Clone, Copy)]
#[allow(non_snake_case)]
pub struct LineDecorationInstance {
    pub aTaskRect: [f32; 4],
    pub aLocalSize: [f32; 2],
    pub aStyle: i32,
    pub aOrientation: i32,
    pub aWavyLineThickness: f32,
}

#[derive(Debug, Clone, Copy)]
#[allow(non_snake_case)]
pub struct PrimitiveInstanceData {
    pub aData: [i32; 4],
}

#[derive(Debug, Clone, Copy)]
#[allow(non_snake_case)]
pub struct Vertex {
    pub aPosition: [f32; 3],
}

#[cfg(feature = "pathfinder")]
#[derive(Debug, Clone, Copy)]
#[allow(non_snake_case)]
pub struct VectorStencilInstance {
    pub aFromPosition: [f32; 2],
    pub aCtrlPosition: [f32; 2],
    pub aToPosition: [f32; 2],
    pub aFromNormal: [f32; 2],
    pub aCtrlNormal: [f32; 2],
    pub aToNormal: [f32; 2],
    pub aPathID: i32,
    pub aPad: i32,
}

#[cfg(feature = "pathfinder")]
#[derive(Debug, Clone, Copy)]
#[allow(non_snake_case)]
pub struct VectorCoverInstance {
    pub aTargetRect: [i32; 4],
    pub aStencilOrigin: [i32; 2],
    pub aSubpixel: i32,
    pub aPad: i32,
}
