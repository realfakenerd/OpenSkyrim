use super::dev::*;

/*
<bitflags name="FurnitureEntryPoints" storage="ushort" versions="#BETHESDA#">
    Bethesda Animation. Furniture entry points. It specifies the direction(s) from where the actor is able to enter (and leave) the position.
    <option bit="0" name="Front">front entry point</option>
    <option bit="1" name="Behind">behind entry point</option>
    <option bit="2" name="Right">right entry point</option>
    <option bit="3" name="Left">left entry point</option>
    <option bit="4" name="Up">up entry point - unknown function. Used on some beds in Skyrim, probably for blocking of sleeping position.</option>
</bitflags>
*/

#[derive(Debug, NomLE)]
pub struct FurnitureEntryPoints(u16);

impl FurnitureEntryPoints {
    pub fn has_front(&self) -> bool {
        self.0 & 0b0000_0001 == 0b0000_0001
    }
    pub fn has_behind(&self) -> bool {
        self.0 & 0b0000_0010 == 0b0000_0010
    }
    pub fn has_right(&self) -> bool {
        self.0 & 0b0000_0100 == 0b0000_0100
    }
    pub fn has_left(&self) -> bool {
        self.0 & 0b0000_1000 == 0b0000_1000
    }
    pub fn has_up(&self) -> bool {
        self.0 & 0b0001_0000 == 0b0001_0000
    }
}

/*
<bitflags name="SkyrimShaderPropertyFlags1" storage="uint" prefix="SLSF1" versions="#SKY# #SSE#">
    Skyrim Shader Property Flags 1
    <option bit="0" name="Specular">Enables Specularity</option>
    <option bit="1" name="Skinned">Required For Skinned Meshes.</option>
    <option bit="2" name="Temp_Refraction"></option>
    <option bit="3" name="Vertex_Alpha">Enables using alpha component of vertex colors.</option>
    <option bit="4" name="Greyscale_To_PaletteColor">in EffectShaderProperty</option>
    <option bit="5" name="Greyscale_To_PaletteAlpha">in EffectShaderProperty</option>
    <option bit="6" name="Use_Falloff">Use Falloff value in EffectShaderProperty</option>
    <option bit="7" name="Environment_Mapping">Environment mapping (uses Envmap Scale).</option>
    <option bit="8" name="Recieve_Shadows">Object can recieve shadows.</option>
    <option bit="9" name="Cast_Shadows">Can cast shadows</option>
    <option bit="10" name="Facegen_Detail_Map">Use a face detail map in the 4th texture slot.</option>
    <option bit="11" name="Parallax">Unused?</option>
    <option bit="12" name="Model_Space_Normals">Use Model space normals and an external Specular Map.</option>
    <option bit="13" name="Non_Projective_Shadows"></option>
    <option bit="14" name="Landscape"></option>
    <option bit="15" name="Refraction">Use normal map for refraction effect.</option>
    <option bit="16" name="Fire_Refraction"></option>
    <option bit="17" name="Eye_Environment_Mapping">Eye Environment Mapping (Must use the Eye shader and the model must be skinned)</option>
    <option bit="18" name="Hair_Soft_Lighting">Keeps from going too bright under lights (hair shader only)</option>
    <option bit="19" name="Screendoor_Alpha_Fade"></option>
    <option bit="20" name="Localmap_Hide_Secret">Object and anything it is positioned above will not render on local map view.</option>
    <option bit="21" name="FaceGen_RGB_Tint">Use tintmask for Face.</option>
    <option bit="22" name="Own_Emit">Provides its own emittance color. (will not absorb light/ambient color?)</option>
    <option bit="23" name="Projected_UV">Used for decalling?</option>
    <option bit="24" name="Multiple_Textures"></option>
    <option bit="25" name="Remappable_Textures"></option>
    <option bit="26" name="Decal"></option>
    <option bit="27" name="Dynamic_Decal"></option>
    <option bit="28" name="Parallax_Occlusion"></option>
    <option bit="29" name="External_Emittance"></option>
    <option bit="30" name="Soft_Effect"></option>
    <option bit="31" name="ZBuffer_Test">ZBuffer Test (1=on)</option>
</bitflags>
*/

#[derive(Debug, NomLE)]
pub struct SkyrimShaderPropertyFlags1(u32);

impl SkyrimShaderPropertyFlags1 {}

/*
<bitflags name="SkyrimShaderPropertyFlags2" storage="uint" prefix="SLSF2" versions="#SKY# #SSE#">
    Skyrim Shader Property Flags 2
    <option bit="0" name="ZBuffer_Write">Enables writing to the Z-Buffer</option>
    <option bit="1" name="LOD_Landscape"></option>
    <option bit="2" name="LOD_Objects"></option>
    <option bit="3" name="No_Fade"></option>
    <option bit="4" name="Double_Sided">Double-sided rendering.</option>
    <option bit="5" name="Vertex_Colors">Has Vertex Colors.</option>
    <option bit="6" name="Glow_Map">Use Glow Map in the third texture slot.</option>
    <option bit="7" name="Assume_Shadowmask"></option>
    <option bit="8" name="Packed_Tangent"></option>
    <option bit="9" name="Multi_Index_Snow"></option>
    <option bit="10" name="Vertex_Lighting"></option>
    <option bit="11" name="Uniform_Scale"></option>
    <option bit="12" name="Fit_Slope"></option>
    <option bit="13" name="Billboard"></option>
    <option bit="14" name="No_LOD_Land_Blend"></option>
    <option bit="15" name="EnvMap_Light_Fade"></option>
    <option bit="16" name="Wireframe">Wireframe (Seems to only work on particles)</option>
    <option bit="17" name="Weapon_Blood">Used for blood decals on weapons.</option>
    <option bit="18" name="Hide_On_Local_Map">Similar to hide secret, but only for self?</option>
    <option bit="19" name="Premult_Alpha">Has Premultiplied Alpha</option>
    <option bit="20" name="Cloud_LOD"></option>
    <option bit="21" name="Anisotropic_Lighting">Hair only?</option>
    <option bit="22" name="No_Transparency_Multisampling"></option>
    <option bit="23" name="Unused01">Unused?</option>
    <option bit="24" name="Multi_Layer_Parallax">Use Multilayer (inner-layer) Map</option>
    <option bit="25" name="Soft_Lighting">Use Soft Lighting Map</option>
    <option bit="26" name="Rim_Lighting">Use Rim Lighting Map</option>
    <option bit="27" name="Back_Lighting">Use Back Lighting Map</option>
    <option bit="28" name="Unused02">Unused?</option>
    <option bit="29" name="Tree_Anim">Enables Vertex Animation, Flutter Animation</option>
    <option bit="30" name="Effect_Lighting"></option>
    <option bit="31" name="HD_LOD_Objects"></option>
</bitflags>
*/

#[derive(Debug, NomLE)]
pub struct SkyrimShaderPropertyFlags2(u32);

/*
<bitflags name="BSValueNodeFlags" storage="byte" versions="#FO3_AND_LATER#">
    Flags for BSValueNode.
    <option bit="0" name="BillboardWorldZ" />
    <option bit="1" name="UsePlayerAdjust" />
</bitflags>
*/

#[derive(Debug, NomLE)]
pub struct BSValueNodeFlags(u8);

/*
<bitflags name="WaterShaderPropertyFlags" storage="uint" prefix="BSWSP" versions="#SKY_AND_LATER#">
        Skyrim water shader property flags
        <option bit="0" name="DISPLACEMENT" />
        <option bit="1" name="LOD" />
        <option bit="2" name="DEPTH" />
        <option bit="3" name="ACTOR_IN_WATER" />
        <option bit="4" name="ACTOR_IN_WATER_IS_MOVING" />
        <option bit="5" name="UNDERWATER" />
        <option bit="6" name="REFLECTIONS" />
        <option bit="7" name="REFRACTIONS" />
        <option bit="8" name="VERTEX_UV" />
        <option bit="9" name="VERTEX_ALPHA_DEPTH" />
        <option bit="10" name="PROCEDURAL" />
        <option bit="11" name="FOG" />
        <option bit="12" name="UPDATE_CONSTANTS" />
        <option bit="13" name="CUBEMAP" />
    </bitflags>
*/

#[derive(Debug, NomLE)]
pub struct WaterShaderPropertyFlags(u32);

/*
<bitfield name="AlphaFlags" storage="ushort">
    Flags for NiAlphaProperty
    <member width="1" pos="0" mask="0x0001" name="Alpha Blend" type="bool" />
    <member width="4" pos="1" mask="0x001E" name="Source Blend Mode" type="AlphaFunction" default="SRC_ALPHA" />
    <member width="4" pos="5" mask="0x01E0" name="Destination Blend Mode" type="AlphaFunction" default="INV_SRC_ALPHA" />
    <member width="1" pos="9" mask="0x0200" name="Alpha Test" type="bool" default="true" />
    <member width="3" pos="10" mask="0x1C00" name="Test Func" type="TestFunction" default="TEST_GREATER" />
    <member width="1" pos="13" mask="0x2000" name="No Sorter" type="bool" />
    <member width="1" pos="14" mask="0x4000" name="Clone Unique" type="bool">Bethesda-only. Always true for weapon blood after FO3.</member>
    <member width="1" pos="15" mask="0x8000" name="Editor Alpha Threshold" type="bool">Bethesda-only. True if the Alpha Threshold is externally controlled.</member>
</bitfield>
*/

#[derive(Debug, NomLE)]
pub struct AlphaFlags(u16);

/*
<bitflags name="InterpBlendFlags" storage="byte">
    Flags for NiBlendInterpolator
    <option bit="0" name="Manager Controlled" />
    <option bit="1" name="Use Only Highest Weight" />
</bitflags>
*/

#[derive(Debug, NomLE)]
pub struct InterpBlendFlags(u8);

/*
<bitflags name="NiSwitchFlags" storage="ushort">
    Flags for NiSwitchNode.
    <option bit="0" name="UpdateOnlyActiveChild">Update Only Active Child</option>
    <option bit="1" name="UpdateControllers">Update Controllers</option>
</bitflags>
*/

#[derive(Debug, NomLE)]
pub struct NiSwitchFlags(u16);
