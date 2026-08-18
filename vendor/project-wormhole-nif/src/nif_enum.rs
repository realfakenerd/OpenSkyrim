use super::dev::*;

/// Describes how the vertex colors are blended with the filtered texture color.

#[repr(u32)]
#[derive(Debug, NomLE)]
pub enum ApplyMode {
    /// Replaces existing color
    Replace = 0,
    /// For placing images on the object like stickers.
    Decal = 1,
    /// Modulates existing color. (Default)
    Modulate = 2,
    /// PS2 Only.  Function Unknown.
    Hilight = 3,
    /// Parallax Flag in some Oblivion meshes.
    Hilight2 = 4,
}

/// The type of texture.

#[repr(u32)]
#[derive(Debug, NomLE)]
pub enum TexType {
    /// The basic texture used by most meshes.
    BaseMap = 0,
    /// Used to darken the model with false lighting.
    DarkMap = 1,
    /// Combined with base map for added detail.  Usually tiled over the mesh many times for close-up view.
    DetailMap = 2,
    /// Allows the specularity (glossyness) of an object to differ across its surface.
    GlossMap = 3,
    /// Creates a glowing effect.  Basically an incandescence map.
    GlowMap = 4,
    /// Used to make the object appear to have more detail than it really does.
    BumpMap = 5,
    /// Used to make the object appear to have more detail than it really does.
    NormalMap = 6,
    /// Parallax map.
    ParallaxMap = 7,
    /// For placing images on the object like stickers.
    Decal0Map = 8,
    /// For placing images on the object like stickers.
    Decal1Map = 9,
    /// For placing images on the object like stickers.
    Decal2Map = 10,
    /// For placing images on the object like stickers.
    Decal3Map = 11,
}

#[repr(u32)]
#[derive(Debug, NomLE)]

pub enum KeyType {
    /// Use linear interpolation.
    LinearKey = 1,
    /// Use quadratic interpolation.  Forward and back tangents will be stored.
    QuadraticKey = 2,
    /// Use Tension Bias Continuity interpolation.  Tension, bias, and continuity will be stored.
    TbcKey = 3,
    /// For use only with rotation data.  Separate X, Y, and Z keys will be stored instead of using quaternions.
    XyzRotationKey = 4,
    /// Step function. Used for visibility keys in NiBoolData.
    ConstKey = 5,
}

/// Bethesda Havok. Material descriptor for a Havok shape in Oblivion.

#[repr(u32)]
#[derive(Debug, NomLE)]
pub enum OblivionHavokMaterial {
    /// Stone
    ObHavMatStone = 0,
    /// Cloth
    ObHavMatCloth = 1,
    /// Dirt
    ObHavMatDirt = 2,
    /// Glass
    ObHavMatGlass = 3,
    /// Grass
    ObHavMatGrass = 4,
    /// Metal
    ObHavMatMetal = 5,
    /// Organic
    ObHavMatOrganic = 6,
    /// Skin
    ObHavMatSkin = 7,
    /// Water
    ObHavMatWater = 8,
    /// Wood
    ObHavMatWood = 9,
    /// Heavy Stone
    ObHavMatHeavyStone = 10,
    /// Heavy Metal
    ObHavMatHeavyMetal = 11,
    /// Heavy Wood
    ObHavMatHeavyWood = 12,
    /// Chain
    ObHavMatChain = 13,
    /// Snow
    ObHavMatSnow = 14,
    /// Stone Stairs
    ObHavMatStoneStairs = 15,
    /// Cloth Stairs
    ObHavMatClothStairs = 16,
    /// Dirt Stairs
    ObHavMatDirtStairs = 17,
    /// Glass Stairs
    ObHavMatGlassStairs = 18,
    /// Grass Stairs
    ObHavMatGrassStairs = 19,
    /// Metal Stairs
    ObHavMatMetalStairs = 20,
    /// Organic Stairs
    ObHavMatOrganicStairs = 21,
    /// Skin Stairs
    ObHavMatSkinStairs = 22,
    /// Water Stairs
    ObHavMatWaterStairs = 23,
    /// Wood Stairs
    ObHavMatWoodStairs = 24,
    /// Heavy Stone Stairs
    ObHavMatHeavyStoneStairs = 25,
    /// Heavy Metal Stairs
    ObHavMatHeavyMetalStairs = 26,
    /// Heavy Wood Stairs
    ObHavMatHeavyWoodStairs = 27,
    /// Chain Stairs
    ObHavMatChainStairs = 28,
    /// Snow Stairs
    ObHavMatSnowStairs = 29,
    /// Elevator
    ObHavMatElevator = 30,
    /// Rubber
    ObHavMatRubber = 31,
}

/*
<enum name="Fallout3HavokMaterial" storage="uint" versions="#BETHESDA#">
    Bethesda Havok. Material descriptor for a Havok shape in Fallout 3 and Fallout NV.
    <option value="0" name="FO_HAV_MAT_STONE">Stone</option>
    <option value="1" name="FO_HAV_MAT_CLOTH">Cloth</option>
    <option value="2" name="FO_HAV_MAT_DIRT">Dirt</option>
    <option value="3" name="FO_HAV_MAT_GLASS">Glass</option>
    <option value="4" name="FO_HAV_MAT_GRASS">Grass</option>
    <option value="5" name="FO_HAV_MAT_METAL">Metal</option>
    <option value="6" name="FO_HAV_MAT_ORGANIC">Organic</option>
    <option value="7" name="FO_HAV_MAT_SKIN">Skin</option>
    <option value="8" name="FO_HAV_MAT_WATER">Water</option>
    <option value="9" name="FO_HAV_MAT_WOOD">Wood</option>
    <option value="10" name="FO_HAV_MAT_HEAVY_STONE">Heavy Stone</option>
    <option value="11" name="FO_HAV_MAT_HEAVY_METAL">Heavy Metal</option>
    <option value="12" name="FO_HAV_MAT_HEAVY_WOOD">Heavy Wood</option>
    <option value="13" name="FO_HAV_MAT_CHAIN">Chain</option>
    <option value="14" name="FO_HAV_MAT_BOTTLECAP">Bottlecap</option>
    <option value="15" name="FO_HAV_MAT_ELEVATOR">Elevator</option>
    <option value="16" name="FO_HAV_MAT_HOLLOW_METAL">Hollow Metal</option>
    <option value="17" name="FO_HAV_MAT_SHEET_METAL">Sheet Metal</option>
    <option value="18" name="FO_HAV_MAT_SAND">Sand</option>
    <option value="19" name="FO_HAV_MAT_BROKEN_CONCRETE">Broken Concrete</option>
    <option value="20" name="FO_HAV_MAT_VEHICLE_BODY">Vehicle Body</option>
    <option value="21" name="FO_HAV_MAT_VEHICLE_PART_SOLID">Vehicle Part Solid</option>
    <option value="22" name="FO_HAV_MAT_VEHICLE_PART_HOLLOW">Vehicle Part Hollow</option>
    <option value="23" name="FO_HAV_MAT_BARREL">Barrel</option>
    <option value="24" name="FO_HAV_MAT_BOTTLE">Bottle</option>
    <option value="25" name="FO_HAV_MAT_SODA_CAN">Soda Can</option>
    <option value="26" name="FO_HAV_MAT_PISTOL">Pistol</option>
    <option value="27" name="FO_HAV_MAT_RIFLE">Rifle</option>
    <option value="28" name="FO_HAV_MAT_SHOPPING_CART">Shopping Cart</option>
    <option value="29" name="FO_HAV_MAT_LUNCHBOX">Lunchbox</option>
    <option value="30" name="FO_HAV_MAT_BABY_RATTLE">Baby Rattle</option>
    <option value="31" name="FO_HAV_MAT_RUBBER_BALL">Rubber Ball</option>
    <option value="32" name="FO_HAV_MAT_STONE_PLATFORM">Stone</option>
    <option value="33" name="FO_HAV_MAT_CLOTH_PLATFORM">Cloth</option>
    <option value="34" name="FO_HAV_MAT_DIRT_PLATFORM">Dirt</option>
    <option value="35" name="FO_HAV_MAT_GLASS_PLATFORM">Glass</option>
    <option value="36" name="FO_HAV_MAT_GRASS_PLATFORM">Grass</option>
    <option value="37" name="FO_HAV_MAT_METAL_PLATFORM">Metal</option>
    <option value="38" name="FO_HAV_MAT_ORGANIC_PLATFORM">Organic</option>
    <option value="39" name="FO_HAV_MAT_SKIN_PLATFORM">Skin</option>
    <option value="40" name="FO_HAV_MAT_WATER_PLATFORM">Water</option>
    <option value="41" name="FO_HAV_MAT_WOOD_PLATFORM">Wood</option>
    <option value="42" name="FO_HAV_MAT_HEAVY_STONE_PLATFORM">Heavy Stone</option>
    <option value="43" name="FO_HAV_MAT_HEAVY_METAL_PLATFORM">Heavy Metal</option>
    <option value="44" name="FO_HAV_MAT_HEAVY_WOOD_PLATFORM">Heavy Wood</option>
    <option value="45" name="FO_HAV_MAT_CHAIN_PLATFORM">Chain</option>
    <option value="46" name="FO_HAV_MAT_BOTTLECAP_PLATFORM">Bottlecap</option>
    <option value="47" name="FO_HAV_MAT_ELEVATOR_PLATFORM">Elevator</option>
    <option value="48" name="FO_HAV_MAT_HOLLOW_METAL_PLATFORM">Hollow Metal</option>
    <option value="49" name="FO_HAV_MAT_SHEET_METAL_PLATFORM">Sheet Metal</option>
    <option value="50" name="FO_HAV_MAT_SAND_PLATFORM">Sand</option>
    <option value="51" name="FO_HAV_MAT_BROKEN_CONCRETE_PLATFORM">Broken Concrete</option>
    <option value="52" name="FO_HAV_MAT_VEHICLE_BODY_PLATFORM">Vehicle Body</option>
    <option value="53" name="FO_HAV_MAT_VEHICLE_PART_SOLID_PLATFORM">Vehicle Part Solid</option>
    <option value="54" name="FO_HAV_MAT_VEHICLE_PART_HOLLOW_PLATFORM">Vehicle Part Hollow</option>
    <option value="55" name="FO_HAV_MAT_BARREL_PLATFORM">Barrel</option>
    <option value="56" name="FO_HAV_MAT_BOTTLE_PLATFORM">Bottle</option>
    <option value="57" name="FO_HAV_MAT_SODA_CAN_PLATFORM">Soda Can</option>
    <option value="58" name="FO_HAV_MAT_PISTOL_PLATFORM">Pistol</option>
    <option value="59" name="FO_HAV_MAT_RIFLE_PLATFORM">Rifle</option>
    <option value="60" name="FO_HAV_MAT_SHOPPING_CART_PLATFORM">Shopping Cart</option>
    <option value="61" name="FO_HAV_MAT_LUNCHBOX_PLATFORM">Lunchbox</option>
    <option value="62" name="FO_HAV_MAT_BABY_RATTLE_PLATFORM">Baby Rattle</option>
    <option value="63" name="FO_HAV_MAT_RUBBER_BALL_PLATFORM">Rubber Ball</option>
    <option value="64" name="FO_HAV_MAT_STONE_STAIRS">Stone</option>
    <option value="65" name="FO_HAV_MAT_CLOTH_STAIRS">Cloth</option>
    <option value="66" name="FO_HAV_MAT_DIRT_STAIRS">Dirt</option>
    <option value="67" name="FO_HAV_MAT_GLASS_STAIRS">Glass</option>
    <option value="68" name="FO_HAV_MAT_GRASS_STAIRS">Grass</option>
    <option value="69" name="FO_HAV_MAT_METAL_STAIRS">Metal</option>
    <option value="70" name="FO_HAV_MAT_ORGANIC_STAIRS">Organic</option>
    <option value="71" name="FO_HAV_MAT_SKIN_STAIRS">Skin</option>
    <option value="72" name="FO_HAV_MAT_WATER_STAIRS">Water</option>
    <option value="73" name="FO_HAV_MAT_WOOD_STAIRS">Wood</option>
    <option value="74" name="FO_HAV_MAT_HEAVY_STONE_STAIRS">Heavy Stone</option>
    <option value="75" name="FO_HAV_MAT_HEAVY_METAL_STAIRS">Heavy Metal</option>
    <option value="76" name="FO_HAV_MAT_HEAVY_WOOD_STAIRS">Heavy Wood</option>
    <option value="77" name="FO_HAV_MAT_CHAIN_STAIRS">Chain</option>
    <option value="78" name="FO_HAV_MAT_BOTTLECAP_STAIRS">Bottlecap</option>
    <option value="79" name="FO_HAV_MAT_ELEVATOR_STAIRS">Elevator</option>
    <option value="80" name="FO_HAV_MAT_HOLLOW_METAL_STAIRS">Hollow Metal</option>
    <option value="81" name="FO_HAV_MAT_SHEET_METAL_STAIRS">Sheet Metal</option>
    <option value="82" name="FO_HAV_MAT_SAND_STAIRS">Sand</option>
    <option value="83" name="FO_HAV_MAT_BROKEN_CONCRETE_STAIRS">Broken Concrete</option>
    <option value="84" name="FO_HAV_MAT_VEHICLE_BODY_STAIRS">Vehicle Body</option>
    <option value="85" name="FO_HAV_MAT_VEHICLE_PART_SOLID_STAIRS">Vehicle Part Solid</option>
    <option value="86" name="FO_HAV_MAT_VEHICLE_PART_HOLLOW_STAIRS">Vehicle Part Hollow</option>
    <option value="87" name="FO_HAV_MAT_BARREL_STAIRS">Barrel</option>
    <option value="88" name="FO_HAV_MAT_BOTTLE_STAIRS">Bottle</option>
    <option value="89" name="FO_HAV_MAT_SODA_CAN_STAIRS">Soda Can</option>
    <option value="90" name="FO_HAV_MAT_PISTOL_STAIRS">Pistol</option>
    <option value="91" name="FO_HAV_MAT_RIFLE_STAIRS">Rifle</option>
    <option value="92" name="FO_HAV_MAT_SHOPPING_CART_STAIRS">Shopping Cart</option>
    <option value="93" name="FO_HAV_MAT_LUNCHBOX_STAIRS">Lunchbox</option>
    <option value="94" name="FO_HAV_MAT_BABY_RATTLE_STAIRS">Baby Rattle</option>
    <option value="95" name="FO_HAV_MAT_RUBBER_BALL_STAIRS">Rubber Ball</option>
    <option value="96" name="FO_HAV_MAT_STONE_STAIRS_PLATFORM">Stone</option>
    <option value="97" name="FO_HAV_MAT_CLOTH_STAIRS_PLATFORM">Cloth</option>
    <option value="98" name="FO_HAV_MAT_DIRT_STAIRS_PLATFORM">Dirt</option>
    <option value="99" name="FO_HAV_MAT_GLASS_STAIRS_PLATFORM">Glass</option>
    <option value="100" name="FO_HAV_MAT_GRASS_STAIRS_PLATFORM">Grass</option>
    <option value="101" name="FO_HAV_MAT_METAL_STAIRS_PLATFORM">Metal</option>
    <option value="102" name="FO_HAV_MAT_ORGANIC_STAIRS_PLATFORM">Organic</option>
    <option value="103" name="FO_HAV_MAT_SKIN_STAIRS_PLATFORM">Skin</option>
    <option value="104" name="FO_HAV_MAT_WATER_STAIRS_PLATFORM">Water</option>
    <option value="105" name="FO_HAV_MAT_WOOD_STAIRS_PLATFORM">Wood</option>
    <option value="106" name="FO_HAV_MAT_HEAVY_STONE_STAIRS_PLATFORM">Heavy Stone</option>
    <option value="107" name="FO_HAV_MAT_HEAVY_METAL_STAIRS_PLATFORM">Heavy Metal</option>
    <option value="108" name="FO_HAV_MAT_HEAVY_WOOD_STAIRS_PLATFORM">Heavy Wood</option>
    <option value="109" name="FO_HAV_MAT_CHAIN_STAIRS_PLATFORM">Chain</option>
    <option value="110" name="FO_HAV_MAT_BOTTLECAP_STAIRS_PLATFORM">Bottlecap</option>
    <option value="111" name="FO_HAV_MAT_ELEVATOR_STAIRS_PLATFORM">Elevator</option>
    <option value="112" name="FO_HAV_MAT_HOLLOW_METAL_STAIRS_PLATFORM">Hollow Metal</option>
    <option value="113" name="FO_HAV_MAT_SHEET_METAL_STAIRS_PLATFORM">Sheet Metal</option>
    <option value="114" name="FO_HAV_MAT_SAND_STAIRS_PLATFORM">Sand</option>
    <option value="115" name="FO_HAV_MAT_BROKEN_CONCRETE_STAIRS_PLATFORM">Broken Concrete</option>
    <option value="116" name="FO_HAV_MAT_VEHICLE_BODY_STAIRS_PLATFORM">Vehicle Body</option>
    <option value="117" name="FO_HAV_MAT_VEHICLE_PART_SOLID_STAIRS_PLATFORM">Vehicle Part Solid</option>
    <option value="118" name="FO_HAV_MAT_VEHICLE_PART_HOLLOW_STAIRS_PLATFORM">Vehicle Part Hollow</option>
    <option value="119" name="FO_HAV_MAT_BARREL_STAIRS_PLATFORM">Barrel</option>
    <option value="120" name="FO_HAV_MAT_BOTTLE_STAIRS_PLATFORM">Bottle</option>
    <option value="121" name="FO_HAV_MAT_SODA_CAN_STAIRS_PLATFORM">Soda Can</option>
    <option value="122" name="FO_HAV_MAT_PISTOL_STAIRS_PLATFORM">Pistol</option>
    <option value="123" name="FO_HAV_MAT_RIFLE_STAIRS_PLATFORM">Rifle</option>
    <option value="124" name="FO_HAV_MAT_SHOPPING_CART_STAIRS_PLATFORM">Shopping Cart</option>
    <option value="125" name="FO_HAV_MAT_LUNCHBOX_STAIRS_PLATFORM">Lunchbox</option>
    <option value="126" name="FO_HAV_MAT_BABY_RATTLE_STAIRS_PLATFORM">Baby Rattle</option>
    <option value="127" name="FO_HAV_MAT_RUBBER_BALL_STAIRS_PLATFORM">Rubber Ball</option>
</enum>
*/

/*
<enum name="SkyrimHavokMaterial" storage="uint" versions="#SKY_AND_LATER#">
    Bethesda Havok. Material descriptor for a Havok shape in Skyrim. CRC32 of the lowercase of the Creation Kit Material Name.
    <option value="0" name="SKY_HAV_MAT_NONE">Invalid Material</option>
    <option value="131151687" name="SKY_HAV_MAT_BROKEN_STONE">Broken Stone</option>
    <option value="322207473" name="SKY_HAV_MAT_MATERIAL_CARRIAGE_WHEEL">Material Carriage Wheel</option>
    <option value="346811165" name="SKY_HAV_MAT_MATERIAL_METAL_LIGHT">Material Metal Light</option>
    <option value="365420259" name="SKY_HAV_MAT_LIGHT_WOOD">Light Wood</option>
    <option value="398949039" name="SKY_HAV_MAT_SNOW">Snow</option>
    <option value="428587608" name="SKY_HAV_MAT_GRAVEL">Gravel</option>
    <option value="438912228" name="SKY_HAV_MAT_MATERIAL_CHAIN_METAL">Material Chain Metal</option>
    <option value="493553910" name="SKY_HAV_MAT_BOTTLE">Bottle</option>
    <option value="500811281" name="SKY_HAV_MAT_WOOD">Wood</option>
    <option value="591247106" name="SKY_HAV_MAT_SKIN">Skin</option>
    <option value="617099282" name="SKY_HAV_MAT_UNKNOWN_617099282">Unknown in Creation Kit v1.9.32.0. Found in Dawnguard DLC in meshes\dlc01\clutter\dlc01deerskin.nif.</option>
    <option value="732141076" name="SKY_HAV_MAT_BARREL">Barrel</option>
    <option value="781661019" name="SKY_HAV_MAT_MATERIAL_CERAMIC_MEDIUM">Material Ceramic Medium</option>
    <option value="790784366" name="SKY_HAV_MAT_MATERIAL_BASKET">Material Basket</option>
    <option value="873356572" name="SKY_HAV_MAT_ICE">Ice</option>
    <option value="880200008" name="SKY_HAV_MAT_STAIRS_GLASS">Stairs Glass</option>
    <option value="899511101" name="SKY_HAV_MAT_STAIRS_STONE">Stairs Stone</option>
    <option value="1024582599" name="SKY_HAV_MAT_WATER">Water</option>
    <option value="1028101969" name="SKY_HAV_MAT_UNKNOWN_1028101969">Unknown in Creation Kit v1.6.89.0. Found in actors\draugr\character assets\skeletons.nif.</option>
    <option value="1060167844" name="SKY_HAV_MAT_MATERIAL_BLADE_1HAND">Material Blade 1 Hand</option>
    <option value="1264672850" name="SKY_HAV_MAT_MATERIAL_BOOK">Material Book</option>
    <option value="1286705471" name="SKY_HAV_MAT_MATERIAL_CARPET">Material Carpet</option>
    <option value="1288358971" name="SKY_HAV_MAT_SOLID_METAL">Solid Metal</option>
    <option value="1305674443" name="SKY_HAV_MAT_MATERIAL_AXE_1HAND">Material Axe 1Hand</option>
    <option value="1440721808" name="SKY_HAV_MAT_UNKNOWN_1440721808">Unknown in Creation Kit v1.6.89.0. Found in armor\draugr\draugrbootsfemale_go.nif or armor\amuletsandrings\amuletgnd.nif.</option>
    <option value="1461712277" name="SKY_HAV_MAT_STAIRS_WOOD">Stairs Wood</option>
    <option value="1486385281" name="SKY_HAV_MAT_MUD">Mud</option>
    <option value="1550912982" name="SKY_HAV_MAT_MATERIAL_BOULDER_SMALL">Material Boulder Small</option>
    <option value="1560365355" name="SKY_HAV_MAT_STAIRS_SNOW">Stairs Snow</option>
    <option value="1570821952" name="SKY_HAV_MAT_HEAVY_STONE">Heavy Stone</option>
    <option value="1574477864" name="SKY_HAV_MAT_UNKNOWN_1574477864">Unknown in Creation Kit v1.6.89.0. Found in actors\dragon\character assets\skeleton.nif.</option>
    <option value="1591009235" name="SKY_HAV_MAT_UNKNOWN_1591009235">Unknown in Creation Kit v1.6.89.0. Found in trap objects or clutter\displaycases\displaycaselgangled01.nif or actors\deer\character assets\skeleton.nif.</option>
    <option value="1607128641" name="SKY_HAV_MAT_MATERIAL_BOWS_STAVES">Material Bows Staves</option>
    <option value="1803571212" name="SKY_HAV_MAT_MATERIAL_WOOD_AS_STAIRS">Material Wood As Stairs</option>
    <option value="1848600814" name="SKY_HAV_MAT_GRASS">Grass</option>
    <option value="1885326971" name="SKY_HAV_MAT_MATERIAL_BOULDER_LARGE">Material Boulder Large</option>
    <option value="1886078335" name="SKY_HAV_MAT_MATERIAL_STONE_AS_STAIRS">Material Stone As Stairs</option>
    <option value="2022742644" name="SKY_HAV_MAT_MATERIAL_BLADE_2HAND">Material Blade 2Hand</option>
    <option value="2025794648" name="SKY_HAV_MAT_MATERIAL_BOTTLE_SMALL">Material Bottle Small</option>
    <option value="2168343821" name="SKY_HAV_MAT_SAND">Sand</option>
    <option value="2229413539" name="SKY_HAV_MAT_HEAVY_METAL">Heavy Metal</option>
    <option value="2290050264" name="SKY_HAV_MAT_UNKNOWN_2290050264">Unknown in Creation Kit v1.9.32.0. Found in Dawnguard DLC in meshes\dlc01\clutter\dlc01sabrecatpelt.nif.</option>
    <option value="2518321175" name="SKY_HAV_MAT_DRAGON">Dragon</option>
    <option value="2617944780" name="SKY_HAV_MAT_MATERIAL_BLADE_1HAND_SMALL">Material Blade 1Hand Small</option>
    <option value="2632367422" name="SKY_HAV_MAT_MATERIAL_SKIN_SMALL">Material Skin Small</option>
    <option value="2742858142" name="SKY_HAV_MAT_MATERIAL_POTS_PANS">Material Pots Pans</option>
    <option value="2892392795" name="SKY_HAV_MAT_STAIRS_BROKEN_STONE">Stairs Broken Stone</option>
    <option value="2965929619" name="SKY_HAV_MAT_MATERIAL_SKIN_LARGE">Material Skin Large</option>
    <option value="2974920155" name="SKY_HAV_MAT_ORGANIC">Organic</option>
    <option value="3049421844" name="SKY_HAV_MAT_MATERIAL_BONE">Material Bone</option>
    <option value="3070783559" name="SKY_HAV_MAT_HEAVY_WOOD">Heavy Wood</option>
    <option value="3074114406" name="SKY_HAV_MAT_MATERIAL_CHAIN">Material Chain</option>
    <option value="3106094762" name="SKY_HAV_MAT_DIRT">Dirt</option>
    <option value="3387452107" name="SKY_HAV_MAT_MATERIAL_SKIN_METAL_LARGE">Material Skin Metal Large</option>
    <option value="3424720541" name="SKY_HAV_MAT_MATERIAL_ARMOR_LIGHT">Material Armor Light</option>
    <option value="3448167928" name="SKY_HAV_MAT_MATERIAL_SHIELD_LIGHT">Material Shield Light</option>
    <option value="3589100606" name="SKY_HAV_MAT_MATERIAL_COIN">Material Coin</option>
    <option value="3702389584" name="SKY_HAV_MAT_MATERIAL_SHIELD_HEAVY">Material Shield Heavy</option>
    <option value="3708432437" name="SKY_HAV_MAT_MATERIAL_ARMOR_HEAVY">Material Armor Heavy</option>
    <option value="3725505938" name="SKY_HAV_MAT_MATERIAL_ARROW">Material Arrow</option>
    <option value="3739830338" name="SKY_HAV_MAT_GLASS">Glass</option>
    <option value="3741512247" name="SKY_HAV_MAT_STONE">Stone</option>
    <option value="3764646153" name="SKY_HAV_MAT_MATERIAL_WATER_PUDDLE">Material Water Puddle</option>
    <option value="3839073443" name="SKY_HAV_MAT_CLOTH">Cloth</option>
    <option value="3855001958" name="SKY_HAV_MAT_MATERIAL_SKIN_METAL_SMALL">Material Skin Metal Small</option>
    <option value="3895166727" name="SKY_HAV_MAT_WARD">Ward</option>
    <option value="3934839107" name="SKY_HAV_MAT_WEB">Web</option>
    <option value="3969592277" name="SKY_HAV_MAT_MATERIAL_BLUNT_2HAND">Material Blunt 2Hand</option>
    <option value="4239621792" name="SKY_HAV_MAT_UNKNOWN_4239621792">Unknown in Creation Kit v1.9.32.0. Found in Dawnguard DLC in meshes\dlc01\prototype\dlc1protoswingingbridge.nif.</option>
    <option value="4283869410" name="SKY_HAV_MAT_MATERIAL_BOULDER_MEDIUM">Material Boulder Medium</option>
    <option value="2794252627" name="SKY_HAV_MAT_UNKNOWN_2794252627" />
    <option value="1668849266" name="SKY_HAV_MAT_UNKNOWN_1668849266" />
    <option value="1734341287" name="SKY_HAV_MAT_UNKNOWN_1734341287" />
    <option value="3974071006" name="SKY_HAV_MAT_UNKNOWN_3974071006" />
    <option value="3941234649" name="SKY_HAV_MAT_UNKNOWN_3941234649">tfxsteelswordbloody</option>
    <option value="1820198263" name="SKY_HAV_MAT_UNKNOWN_1820198263">steelgreatsword</option>
</enum>

*/

/*
<enum name="OblivionLayer" storage="byte" versions="#BETHESDA#">
    Bethesda Havok. Describes the collision layer a body belongs to in Oblivion.
    <option value="0" name="OL_UNIDENTIFIED">Unidentified (white)</option>
    <option value="1" name="OL_STATIC">Static (red)</option>
    <option value="2" name="OL_ANIM_STATIC">AnimStatic (magenta)</option>
    <option value="3" name="OL_TRANSPARENT">Transparent (light pink)</option>
    <option value="4" name="OL_CLUTTER">Clutter (light blue)</option>
    <option value="5" name="OL_WEAPON">Weapon (orange)</option>
    <option value="6" name="OL_PROJECTILE">Projectile (light orange)</option>
    <option value="7" name="OL_SPELL">Spell (cyan)</option>
    <option value="8" name="OL_BIPED">Biped (green) Seems to apply to all creatures/NPCs</option>
    <option value="9" name="OL_TREES">Trees (light brown)</option>
    <option value="10" name="OL_PROPS">Props (magenta)</option>
    <option value="11" name="OL_WATER">Water (cyan)</option>
    <option value="12" name="OL_TRIGGER">Trigger (light grey)</option>
    <option value="13" name="OL_TERRAIN">Terrain (light yellow)</option>
    <option value="14" name="OL_TRAP">Trap (light grey)</option>
    <option value="15" name="OL_NONCOLLIDABLE">NonCollidable (white)</option>
    <option value="16" name="OL_CLOUD_TRAP">CloudTrap (greenish grey)</option>
    <option value="17" name="OL_GROUND">Ground (none)</option>
    <option value="18" name="OL_PORTAL">Portal (green)</option>
    <option value="19" name="OL_STAIRS">Stairs (white)</option>
    <option value="20" name="OL_CHAR_CONTROLLER">CharController (yellow)</option>
    <option value="21" name="OL_AVOID_BOX">AvoidBox (dark yellow)</option>
    <option value="22" name="OL_UNKNOWN1">? (white)</option>
    <option value="23" name="OL_UNKNOWN2">? (white)</option>
    <option value="24" name="OL_CAMERA_PICK">CameraPick (white)</option>
    <option value="25" name="OL_ITEM_PICK">ItemPick (white)</option>
    <option value="26" name="OL_LINE_OF_SIGHT">LineOfSight (white)</option>
    <option value="27" name="OL_PATH_PICK">PathPick (white)</option>
    <option value="28" name="OL_CUSTOM_PICK_1">CustomPick1 (white)</option>
    <option value="29" name="OL_CUSTOM_PICK_2">CustomPick2 (white)</option>
    <option value="30" name="OL_SPELL_EXPLOSION">SpellExplosion (white)</option>
    <option value="31" name="OL_DROPPING_PICK">DroppingPick (white)</option>
    <option value="32" name="OL_OTHER">Other (white)</option>
    <option value="33" name="OL_HEAD">Head</option>
    <option value="34" name="OL_BODY">Body</option>
    <option value="35" name="OL_SPINE1">Spine1</option>
    <option value="36" name="OL_SPINE2">Spine2</option>
    <option value="37" name="OL_L_UPPER_ARM">LUpperArm</option>
    <option value="38" name="OL_L_FOREARM">LForeArm</option>
    <option value="39" name="OL_L_HAND">LHand</option>
    <option value="40" name="OL_L_THIGH">LThigh</option>
    <option value="41" name="OL_L_CALF">LCalf</option>
    <option value="42" name="OL_L_FOOT">LFoot</option>
    <option value="43" name="OL_R_UPPER_ARM">RUpperArm</option>
    <option value="44" name="OL_R_FOREARM">RForeArm</option>
    <option value="45" name="OL_R_HAND">RHand</option>
    <option value="46" name="OL_R_THIGH">RThigh</option>
    <option value="47" name="OL_R_CALF">RCalf</option>
    <option value="48" name="OL_R_FOOT">RFoot</option>
    <option value="49" name="OL_TAIL">Tail</option>
    <option value="50" name="OL_SIDE_WEAPON">SideWeapon</option>
    <option value="51" name="OL_SHIELD">Shield</option>
    <option value="52" name="OL_QUIVER">Quiver</option>
    <option value="53" name="OL_BACK_WEAPON">BackWeapon</option>
    <option value="54" name="OL_BACK_WEAPON2">BackWeapon (?)</option>
    <option value="55" name="OL_PONYTAIL">PonyTail</option>
    <option value="56" name="OL_WING">Wing</option>
    <option value="57" name="OL_NULL">Null</option>
</enum>

*/

/*
<enum name="Fallout3Layer" storage="byte" versions="#BETHESDA#">
    Bethesda Havok. Describes the collision layer a body belongs to in Fallout 3 and Fallout NV.
    <option value="0" name="FOL_UNIDENTIFIED">Unidentified (white)</option>
    <option value="1" name="FOL_STATIC">Static (red)</option>
    <option value="2" name="FOL_ANIM_STATIC">AnimStatic (magenta)</option>
    <option value="3" name="FOL_TRANSPARENT">Transparent (light pink)</option>
    <option value="4" name="FOL_CLUTTER">Clutter (light blue)</option>
    <option value="5" name="FOL_WEAPON">Weapon (orange)</option>
    <option value="6" name="FOL_PROJECTILE">Projectile (light orange)</option>
    <option value="7" name="FOL_SPELL">Spell (cyan)</option>
    <option value="8" name="FOL_BIPED">Biped (green) Seems to apply to all creatures/NPCs</option>
    <option value="9" name="FOL_TREES">Trees (light brown)</option>
    <option value="10" name="FOL_PROPS">Props (magenta)</option>
    <option value="11" name="FOL_WATER">Water (cyan)</option>
    <option value="12" name="FOL_TRIGGER">Trigger (light grey)</option>
    <option value="13" name="FOL_TERRAIN">Terrain (light yellow)</option>
    <option value="14" name="FOL_TRAP">Trap (light grey)</option>
    <option value="15" name="FOL_NONCOLLIDABLE">NonCollidable (white)</option>
    <option value="16" name="FOL_CLOUD_TRAP">CloudTrap (greenish grey)</option>
    <option value="17" name="FOL_GROUND">Ground (none)</option>
    <option value="18" name="FOL_PORTAL">Portal (green)</option>
    <option value="19" name="FOL_DEBRIS_SMALL">DebrisSmall (white)</option>
    <option value="20" name="FOL_DEBRIS_LARGE">DebrisLarge (white)</option>
    <option value="21" name="FOL_ACOUSTIC_SPACE">AcousticSpace (white)</option>
    <option value="22" name="FOL_ACTORZONE">Actorzone (white)</option>
    <option value="23" name="FOL_PROJECTILEZONE">Projectilezone (white)</option>
    <option value="24" name="FOL_GASTRAP">GasTrap (yellowish green)</option>
    <option value="25" name="FOL_SHELLCASING">ShellCasing (white)</option>
    <option value="26" name="FOL_TRANSPARENT_SMALL">TransparentSmall (white)</option>
    <option value="27" name="FOL_INVISIBLE_WALL">InvisibleWall (white)</option>
    <option value="28" name="FOL_TRANSPARENT_SMALL_ANIM">TransparentSmallAnim (white)</option>
    <option value="29" name="FOL_DEADBIP">Dead Biped (green)</option>
    <option value="30" name="FOL_CHARCONTROLLER">CharController (yellow)</option>
    <option value="31" name="FOL_AVOIDBOX">Avoidbox (orange)</option>
    <option value="32" name="FOL_COLLISIONBOX">Collisionbox (white)</option>
    <option value="33" name="FOL_CAMERASPHERE">Camerasphere (white)</option>
    <option value="34" name="FOL_DOORDETECTION">Doordetection (white)</option>
    <option value="35" name="FOL_CAMERAPICK">Camerapick (white)</option>
    <option value="36" name="FOL_ITEMPICK">Itempick (white)</option>
    <option value="37" name="FOL_LINEOFSIGHT">LineOfSight (white)</option>
    <option value="38" name="FOL_PATHPICK">Pathpick (white)</option>
    <option value="39" name="FOL_CUSTOMPICK1">Custompick1 (white)</option>
    <option value="40" name="FOL_CUSTOMPICK2">Custompick2 (white)</option>
    <option value="41" name="FOL_SPELLEXPLOSION">SpellExplosion (white)</option>
    <option value="42" name="FOL_DROPPINGPICK">Droppingpick (white)</option>
    <option value="43" name="FOL_NULL">Null (white)</option>
</enum>

*/

/*
<enum name="SkyrimLayer" storage="byte" versions="#SKY_AND_LATER#">
    Bethesda Havok. Describes the collision layer a body belongs to in Skyrim.
    <option value="0" name="SKYL_UNIDENTIFIED">Unidentified</option>
    <option value="1" name="SKYL_STATIC">Static</option>
    <option value="2" name="SKYL_ANIMSTATIC">Anim Static</option>
    <option value="3" name="SKYL_TRANSPARENT">Transparent</option>
    <option value="4" name="SKYL_CLUTTER">Clutter. Object with this layer will float on water surface.</option>
    <option value="5" name="SKYL_WEAPON">Weapon</option>
    <option value="6" name="SKYL_PROJECTILE">Projectile</option>
    <option value="7" name="SKYL_SPELL">Spell</option>
    <option value="8" name="SKYL_BIPED">Biped. Seems to apply to all creatures/NPCs</option>
    <option value="9" name="SKYL_TREES">Trees</option>
    <option value="10" name="SKYL_PROPS">Props</option>
    <option value="11" name="SKYL_WATER">Water</option>
    <option value="12" name="SKYL_TRIGGER">Trigger</option>
    <option value="13" name="SKYL_TERRAIN">Terrain</option>
    <option value="14" name="SKYL_TRAP">Trap</option>
    <option value="15" name="SKYL_NONCOLLIDABLE">NonCollidable</option>
    <option value="16" name="SKYL_CLOUD_TRAP">CloudTrap</option>
    <option value="17" name="SKYL_GROUND">Ground. It seems that produces no sound when collide.</option>
    <option value="18" name="SKYL_PORTAL">Portal</option>
    <option value="19" name="SKYL_DEBRIS_SMALL">Debris Small</option>
    <option value="20" name="SKYL_DEBRIS_LARGE">Debris Large</option>
    <option value="21" name="SKYL_ACOUSTIC_SPACE">Acoustic Space</option>
    <option value="22" name="SKYL_ACTORZONE">Actor Zone</option>
    <option value="23" name="SKYL_PROJECTILEZONE">Projectile Zone</option>
    <option value="24" name="SKYL_GASTRAP">Gas Trap</option>
    <option value="25" name="SKYL_SHELLCASING">Shell Casing</option>
    <option value="26" name="SKYL_TRANSPARENT_SMALL">Transparent Small</option>
    <option value="27" name="SKYL_INVISIBLE_WALL">Invisible Wall</option>
    <option value="28" name="SKYL_TRANSPARENT_SMALL_ANIM">Transparent Small Anim</option>
    <option value="29" name="SKYL_WARD">Ward</option>
    <option value="30" name="SKYL_CHARCONTROLLER">Char Controller</option>
    <option value="31" name="SKYL_STAIRHELPER">Stair Helper</option>
    <option value="32" name="SKYL_DEADBIP">Dead Bip</option>
    <option value="33" name="SKYL_BIPED_NO_CC">Biped No CC</option>
    <option value="34" name="SKYL_AVOIDBOX">Avoid Box</option>
    <option value="35" name="SKYL_COLLISIONBOX">Collision Box</option>
    <option value="36" name="SKYL_CAMERASHPERE">Camera Sphere</option>
    <option value="37" name="SKYL_DOORDETECTION">Door Detection</option>
    <option value="38" name="SKYL_CONEPROJECTILE">Cone Projectile</option>
    <option value="39" name="SKYL_CAMERAPICK">Camera Pick</option>
    <option value="40" name="SKYL_ITEMPICK">Item Pick</option>
    <option value="41" name="SKYL_LINEOFSIGHT">Line of Sight</option>
    <option value="42" name="SKYL_PATHPICK">Path Pick</option>
    <option value="43" name="SKYL_CUSTOMPICK1">Custom Pick 1</option>
    <option value="44" name="SKYL_CUSTOMPICK2">Custom Pick 2</option>
    <option value="45" name="SKYL_SPELLEXPLOSION">Spell Explosion</option>
    <option value="46" name="SKYL_DROPPINGPICK">Dropping Pick</option>
    <option value="47" name="SKYL_DEADACTORZONE">Dead Actor Zone</option>
    <option value="48" name="SKYL_TRIGGER_FALLINGTRAP">Falling Trap Trigger</option>
    <option value="49" name="SKYL_NAVCUT">Nav Cut</option>
    <option value="50" name="SKYL_CRITTER">Critter</option>
    <option value="51" name="SKYL_SPELLTRIGGER">Spell Trigger</option>
    <option value="52" name="SKYL_LIVING_AND_DEAD_ACTORS">Living And Dead Actors</option>
    <option value="53" name="SKYL_DETECTION">Detection</option>
    <option value="54" name="SKYL_TRAP_TRIGGER">Trap Trigger</option>
</enum>

*/

/*
<enum name="BipedPart" storage="byte">
    <option value="0" name="P_OTHER">Other</option>
    <option value="1" name="P_HEAD">Head</option>
    <option value="2" name="P_BODY">Body</option>
    <option value="3" name="P_SPINE1">Spine1</option>
    <option value="4" name="P_SPINE2">Spine2</option>
    <option value="5" name="P_L_UPPER_ARM">LUpperArm</option>
    <option value="6" name="P_L_FOREARM">LForeArm</option>
    <option value="7" name="P_L_HAND">LHand</option>
    <option value="8" name="P_L_THIGH">LThigh</option>
    <option value="9" name="P_L_CALF">LCalf</option>
    <option value="10" name="P_L_FOOT">LFoot</option>
    <option value="11" name="P_R_UPPER_ARM">RUpperArm</option>
    <option value="12" name="P_R_FOREARM">RForeArm</option>
    <option value="13" name="P_R_HAND">RHand</option>
    <option value="14" name="P_R_THIGH">RThigh</option>
    <option value="15" name="P_R_CALF">RCalf</option>
    <option value="16" name="P_R_FOOT">RFoot</option>
    <option value="17" name="P_TAIL">Tail</option>
    <option value="18" name="P_SHIELD">Shield</option>
    <option value="19" name="P_QUIVER">Quiver</option>
    <option value="20" name="P_WEAPON">Weapon</option>
    <option value="21" name="P_PONYTAIL">Ponytail</option>
    <option value="22" name="P_WING">Wing</option>
    <option value="23" name="P_PACK">Pack</option>
    <option value="24" name="P_CHAIN">Chain</option>
    <option value="25" name="P_ADDON_HEAD">AddonHead</option>
    <option value="26" name="P_ADDON_CHEST">AddonChest</option>
    <option value="27" name="P_ADDON_LEG">AddonLeg</option>
    <option value="28" name="P_ADDON_ARM">AddonArm</option>
</enum>

*/

/*
    <enum name="hkMoppCodeBuildType" storage="byte" versions="#SKY_AND_LATER#">
        hkpMoppCode::BuildType
        A byte describing if MOPP Data is organized into chunks (PS3) or not (PC)
        <option value="0" name="BUILT_WITH_CHUNK_SUBDIVISION">Organized in chunks for PS3.</option>
        <option value="1" name="BUILT_WITHOUT_CHUNK_SUBDIVISION">Not organized in chunks for PC. (Default)</option>
        <option value="2" name="BUILD_NOT_SET">Build type not set yet.</option>
    </enum>
*/

#[repr(u8)]
#[derive(Debug, NomLE)]
pub enum HKMoppCodeBuildType {
    BuiltWithChunkSubdivision = 0,
    BuiltWithoutChunkSubdivision = 1,
    BuildNotSet = 2,
}

/*
<enum name="PlatformID" storage="uint" prefix="PLATFORM">
    Target platform for NiPersistentSrcTextureRendererData (later than 30.1).
    <option value="0" name="ANY" />
    <option value="1" name="XENON" />
    <option value="2" name="PS3" />
    <option value="3" name="DX9" />
    <option value="4" name="WII" />
    <option value="5" name="D3D10" />
    <option value="6" name="UNKNOWN_6" />
    <option value="7" name="UNKNOWN_7" />
    <option value="8" name="UNKNOWN_8" />
</enum>
*/

#[repr(u32)]
#[derive(Debug, NomLE)]
pub enum PlatformID {
    Any = 0,
    Xenon = 1,
    PS3 = 2,
    DX9 = 3,
    Wii = 4,
    D3D10 = 5,
    Unknown6 = 6,
    Unknown7 = 7,
    Unknown8 = 8,
}

/*
<enum name="RendererID" storage="uint" prefix="RENDERER">
    Target renderer for NiPersistentSrcTextureRendererData (until 30.1).
    <option value="0" name="XBOX360" />
    <option value="1" name="PS3" />
    <option value="2" name="DX9" />
    <option value="3" name="D3D10" />
    <option value="4" name="WII" />
    <option value="5" name="GENERIC" />
    <option value="6" name="D3D11" />
</enum>
*/

#[repr(u32)]
#[derive(Debug, NomLE)]
pub enum RendererID {
    XBOX360 = 0,
    PS3 = 1,
    DX9 = 2,
    D3D10 = 3,
    Wii = 4,
    Generic = 5,
    D3D11 = 6,
}

/*
<enum name="PixelFormat" storage="uint" prefix="PX">
    Describes the pixel format used by the NiPixelData object to store a texture.
    <option value="0" name="FMT_RGB">24-bit RGB. 8 bits per red, blue, and green component.</option>
    <option value="1" name="FMT_RGBA">32-bit RGB with alpha. 8 bits per red, blue, green, and alpha component.</option>
    <option value="2" name="FMT_PAL">8-bit palette index.</option>
    <option value="3" name="FMT_PALA">8-bit palette index with alpha.</option>
    <option value="4" name="FMT_DXT1">DXT1 compressed texture.</option>
    <option value="5" name="FMT_DXT3">DXT3 compressed texture.</option>
    <option value="6" name="FMT_DXT5">DXT5 compressed texture.</option>
    <option value="7" name="FMT_RGB24NONINT">(Deprecated) 24-bit noninterleaved texture, an old PS2 format.</option>
    <option value="8" name="FMT_BUMP">Uncompressed dU/dV gradient bump map.</option>
    <option value="9" name="FMT_BUMPLUMA">Uncompressed dU/dV gradient bump map with luma channel representing shininess.</option>
    <option value="10" name="FMT_RENDERSPEC">Generic descriptor for any renderer-specific format not described by other formats.</option>
    <option value="11" name="FMT_1CH">Generic descriptor for formats with 1 component.</option>
    <option value="12" name="FMT_2CH">Generic descriptor for formats with 2 components.</option>
    <option value="13" name="FMT_3CH">Generic descriptor for formats with 3 components.</option>
    <option value="14" name="FMT_4CH">Generic descriptor for formats with 4 components.</option>
    <option value="15" name="FMT_DEPTH_STENCIL">Indicates the NiPixelFormat is meant to be used on a depth/stencil surface.</option>
    <option value="16" name="FMT_UNKNOWN" />
</enum>
*/

#[repr(u32)]
#[derive(Debug, NomLE)]
pub enum PixelFormat {
    RGB = 0,
    RGBA = 1,
    PAL = 2,
    PALA = 3,
    DXT1 = 4,
    DXT3 = 5,
    DXT5 = 6,
    RGB24NONINT = 7,
    BUMP = 8,
    BUMPLUMA = 9,
    RENDERSPEC = 10,
    CH1 = 11,
    CH2 = 12,
    CH3 = 13,
    CH4 = 14,
    DepthStencil = 15,
    Unknown = 16,
}

/*
<enum name="PixelTiling" storage="uint" prefix="PX">
    Describes whether pixels have been tiled from their standard row-major format to a format optimized for a particular platform.
    <option value="0" name="TILE_NONE" />
    <option value="1" name="TILE_XENON" />
    <option value="2" name="TILE_WII" />
    <option value="3" name="TILE_NV_SWIZZLED" />
</enum>
*/

#[repr(u32)]
#[derive(Debug, NomLE)]
pub enum PixelTiling {
    None = 0,
    Xenon = 1,
    Wii = 2,
    NVSwizzled = 3,
}

/*
<enum name="PixelComponent" storage="uint" prefix="PX">
    Describes the pixel format used by the NiPixelData object to store a texture.
    <option value="0" name="COMP_RED" />
    <option value="1" name="COMP_GREEN" />
    <option value="2" name="COMP_BLUE" />
    <option value="3" name="COMP_ALPHA" />
    <option value="4" name="COMP_COMPRESSED" />
    <option value="5" name="COMP_OFFSET_U" />
    <option value="6" name="COMP_OFFSET_V" />
    <option value="7" name="COMP_OFFSET_W" />
    <option value="8" name="COMP_OFFSET_Q" />
    <option value="9" name="COMP_LUMA" />
    <option value="10" name="COMP_HEIGHT" />
    <option value="11" name="COMP_VECTOR_X" />
    <option value="12" name="COMP_VECTOR_Y" />
    <option value="13" name="COMP_VECTOR_Z" />
    <option value="14" name="COMP_PADDING" />
    <option value="15" name="COMP_INTENSITY" />
    <option value="16" name="COMP_INDEX" />
    <option value="17" name="COMP_DEPTH" />
    <option value="18" name="COMP_STENCIL" />
    <option value="19" name="COMP_EMPTY" />
</enum>
*/

#[repr(u32)]
#[derive(Debug, NomLE)]
pub enum PixelComponent {
    Red = 0,
    Green = 1,
    Blue = 2,
    Alpha = 3,
    Compressed = 4,
    OffsetU = 5,
    OffsetV = 6,
    OffsetW = 7,
    OffsetQ = 8,
    Luma = 9,
    Height = 10,
    VectorX = 11,
    VectorY = 12,
    VectorZ = 13,
    Padding = 14,
    Intensity = 15,
    Index = 16,
    Depth = 17,
    Stencil = 18,
    Empty = 19,
}

/*
<enum name="PixelRepresentation" storage="uint" prefix="PX">
    Describes how each pixel should be accessed on NiPixelFormat.
    <option value="0" name="REP_NORM_INT" />
    <option value="1" name="REP_HALF" />
    <option value="2" name="REP_FLOAT" />
    <option value="3" name="REP_INDEX" />
    <option value="4" name="REP_COMPRESSED" />
    <option value="5" name="REP_UNKNOWN" />
    <option value="6" name="REP_INT" />
</enum>
*/

#[repr(u32)]
#[derive(Debug, NomLE)]
pub enum PixelRepresentation {
    NormInt = 0,
    Half = 1,
    Float = 2,
    Index = 3,
    Compressed = 4,
    Unknown = 5,
    Int = 6,
}

/*
<enum name="PixelLayout" storage="uint" prefix="PX">
    Describes the color depth in an NiTexture.
    <option value="0" name="LAY_PALETTIZED_8">Texture is in 8-bit palettized format.</option>
    <option value="1" name="LAY_HIGH_COLOR_16">Texture is in 16-bit high color format.</option>
    <option value="2" name="LAY_TRUE_COLOR_32">Texture is in 32-bit true color format.</option>
    <option value="3" name="LAY_COMPRESSED">Texture is compressed.</option>
    <option value="4" name="LAY_BUMPMAP">Texture is a grayscale bump map.</option>
    <option value="5" name="LAY_PALETTIZED_4">Texture is in 4-bit palettized format.</option>
    <option value="6" name="LAY_DEFAULT">Use default setting.</option>
    <option value="7" name="LAY_SINGLE_COLOR_8" />
    <option value="8" name="LAY_SINGLE_COLOR_16" />
    <option value="9" name="LAY_SINGLE_COLOR_32" />
    <option value="10" name="LAY_DOUBLE_COLOR_32" />
    <option value="11" name="LAY_DOUBLE_COLOR_64" />
    <option value="12" name="LAY_FLOAT_COLOR_32" />
    <option value="13" name="LAY_FLOAT_COLOR_64" />
    <option value="14" name="LAY_FLOAT_COLOR_128" />
    <option value="15" name="LAY_SINGLE_COLOR_4" />
    <option value="16" name="LAY_DEPTH_24_X8" />
</enum>
*/

#[repr(u32)]
#[derive(Debug, NomLE)]

pub enum PixelLayout {
    Palettized8 = 0,
    HighColor16 = 1,
    TrueColor32 = 2,
    Compressed = 3,
    BumpMap = 4,
    Palettized4 = 5,
    Default = 6,
    SingleColor8 = 7,
    SingleColor16 = 8,
    SingleColor32 = 9,
    DoubleColor32 = 10,
    DoubleColor64 = 11,
    FloatColor32 = 12,
    FloatColor64 = 13,
    FloatColor128 = 14,
    SingleColor4 = 15,
    Depth24X8 = 16,
}

/*
<enum name="MipMapFormat" storage="uint">
    Describes how mipmaps are handled in an NiTexture.
    <option value="0" name="MIP_FMT_NO">Texture does not use mip maps.</option>
    <option value="1" name="MIP_FMT_YES">Texture uses mip maps.</option>
    <option value="2" name="MIP_FMT_DEFAULT">Use default setting.</option>
</enum>
*/

#[repr(u32)]
#[derive(Debug, NomLE)]

pub enum MipMapFormat {
    No = 0,
    Yes = 1,
    Default = 2,
}

/*
<enum name="AlphaFormat" storage="uint">
    Describes how transparency is handled in an NiTexture.
    <option value="0" name="ALPHA_NONE">No alpha.</option>
    <option value="1" name="ALPHA_BINARY">1-bit alpha.</option>
    <option value="2" name="ALPHA_SMOOTH">Interpolated 4- or 8-bit alpha.</option>
    <option value="3" name="ALPHA_DEFAULT">Use default setting.</option>
</enum>
*/

#[repr(u32)]
#[derive(Debug, NomLE)]

pub enum AlphaFormat {
    None = 0,
    Binary = 1,
    Smooth = 2,
    Default = 3,
}

/*
<enum name="TexClampMode" storage="uint">
    Describes the availiable texture clamp modes, i.e. the behavior of UV mapping outside the [0,1] range.
    <option value="0" name="CLAMP_S_CLAMP_T">Clamp in both directions.</option>
    <option value="1" name="CLAMP_S_WRAP_T">Clamp in the S(U) direction but wrap in the T(V) direction.</option>
    <option value="2" name="WRAP_S_CLAMP_T">Wrap in the S(U) direction but clamp in the T(V) direction.</option>
    <option value="3" name="WRAP_S_WRAP_T">Wrap in both directions.</option>
</enum>
*/

#[repr(u32)]
#[derive(Debug, NomLE)]

pub enum TexClampMode {
    ClampSClampT = 0,
    ClampSWrapT = 1,
    WrapSClampT = 2,
    WrapSWrapT = 3,
}

/*
<enum name="TexFilterMode" storage="uint">
    Describes the availiable texture filter modes, i.e. the way the pixels in a texture are displayed on screen.
    <option value="0" name="FILTER_NEAREST">Nearest neighbor. Uses nearest texel with no mipmapping.</option>
    <option value="1" name="FILTER_BILERP">Bilinear. Linear interpolation with no mipmapping.</option>
    <option value="2" name="FILTER_TRILERP">Trilinear. Linear intepolation between 8 texels (4 nearest texels between 2 nearest mip levels).</option>
    <option value="3" name="FILTER_NEAREST_MIPNEAREST">Nearest texel on nearest mip level.</option>
    <option value="4" name="FILTER_NEAREST_MIPLERP">Linear interpolates nearest texel between two nearest mip levels.</option>
    <option value="5" name="FILTER_BILERP_MIPNEAREST">Linear interpolates on nearest mip level.</option>
    <option value="6" name="FILTER_ANISOTROPIC">Anisotropic filtering. One or many trilinear samples depending on anisotropy.</option>
</enum>
*/

#[repr(u32)]
#[derive(Debug, NomLE)]

pub enum TexFilterMode {
    Nearest = 0,
    Bilerp = 1,
    Trilerp = 2,
    NearestMipNearest = 3,
    NearestMipLerp = 4,
    BilerpMipNearest = 5,
    Anisotropic = 6,
}

/*
<enum name="SourceVertexMode" storage="uint">
    Describes how to apply vertex colors for NiVertexColorProperty.
    <option value="0" name="VERT_MODE_SRC_IGNORE">Emissive, ambient, and diffuse colors are all specified by the NiMaterialProperty.</option>
    <option value="1" name="VERT_MODE_SRC_EMISSIVE">Emissive colors are specified by the source vertex colors. Ambient+Diffuse are specified by the NiMaterialProperty.</option>
    <option value="2" name="VERT_MODE_SRC_AMB_DIF">Ambient+Diffuse colors are specified by the source vertex colors. Emissive is specified by the NiMaterialProperty. (Default)</option>
</enum>
*/

#[repr(u32)]
#[derive(Debug, NomLE)]

pub enum SourceVertexMode {
    Ignore = 0,
    Emissive = 1,
    AmbDif = 2,
}

/*
<enum name="LightingMode" storage="uint">
    Describes which lighting equation components influence the final vertex color for NiVertexColorProperty.
    <option value="0" name="LIGHT_MODE_EMISSIVE">Emissive.</option>
    <option value="1" name="LIGHT_MODE_EMI_AMB_DIF">Emissive + Ambient + Diffuse. (Default)</option>
</enum>
*/

#[repr(u32)]
#[derive(Debug, NomLE)]

pub enum LightingMode {
    Emissive = 0,
    EmiAmbDif = 1,
}

/*
<enum name="CycleType" storage="uint">
    The animation cyle behavior.
    <option value="0" name="CYCLE_LOOP">Loop</option>
    <option value="1" name="CYCLE_REVERSE">Reverse</option>
    <option value="2" name="CYCLE_CLAMP">Clamp</option>
</enum>
*/

#[repr(u32)]
#[derive(Debug, NomLE)]

pub enum CycleType {
    Loop = 0,
    Reverse = 1,
    Clamp = 2,
}

/*
<enum name="FieldType" storage="uint">
    The force field type.
    <option value="0" name="FIELD_WIND">Wind (fixed direction)</option>
    <option value="1" name="FIELD_POINT">Point (fixed origin)</option>
</enum>
*/

#[repr(u32)]
#[derive(Debug, NomLE)]

pub enum FieldType {
    Wind = 0,
    Point = 1,
}

/*
<enum name="BillboardMode" storage="ushort">
    Determines the way the billboard will react to the camera.
    Billboard mode is stored in lowest 3 bits although Oblivion vanilla nifs uses values higher than 7.
    <option value="0" name="ALWAYS_FACE_CAMERA">Align billboard and camera forward vector. Minimized rotation.</option>
    <option value="1" name="ROTATE_ABOUT_UP">Align billboard and camera forward vector while allowing rotation around the up axis.</option>
    <option value="2" name="RIGID_FACE_CAMERA">Align billboard and camera forward vector. Non-minimized rotation.</option>
    <option value="3" name="ALWAYS_FACE_CENTER">Billboard forward vector always faces camera ceneter. Minimized rotation.</option>
    <option value="4" name="RIGID_FACE_CENTER">Billboard forward vector always faces camera ceneter. Non-minimized rotation.</option>
    <option value="5" name="BSROTATE_ABOUT_UP">The billboard will only rotate around its local Z axis (it always stays in its local X-Y plane).</option>
    <option value="9" name="ROTATE_ABOUT_UP2">The billboard will only rotate around the up axis (same as ROTATE_ABOUT_UP?).</option>
    <option value="8" name="UNKNOWN_8">Found in Civ IV Gravebringer and Gravebringer_FX</option>
    <option value="10" name="UNKNOWN_10">Found in FO3 dlc04lighthouselightmech01</option>
    <option value="11" name="UNKNOWN_11">Found in Civ IV Afterworld_Boss_FX</option>
    <option value="12" name="UNKNOWN_12">Found in IRIS Online etc.</option>
</enum>
*/

#[repr(u16)]
#[derive(Debug, NomLE)]

pub enum BillboardMode {
    AlwaysFaceCamera = 0,
    RotateAboutUp = 1,
    RigidFaceCamera = 2,
    AlwaysFaceCenter = 3,
    RigidFaceCenter = 4,
    BSRotateAboutUp = 5,
    RotateAboutUp2 = 9,
    Unknown8 = 8,
    Unknown10 = 10,
    Unknown11 = 11,
    Unknown12 = 12,
}

/*
<enum name="StencilTestFunc" storage="uint" prefix="STENCIL">
    Describes stencil buffer test modes for NiStencilProperty.
    <option value="0" name="TEST_NEVER">Always false. Ref value is ignored.</option>
    <option value="1" name="TEST_LESS">VRef ‹ VBuf</option>
    <option value="2" name="TEST_EQUAL">VRef = VBuf</option>
    <option value="3" name="TEST_LESS_EQUAL">VRef ≤ VBuf</option>
    <option value="4" name="TEST_GREATER">VRef › VBuf</option>
    <option value="5" name="TEST_NOT_EQUAL">VRef ≠ VBuf</option>
    <option value="6" name="TEST_GREATER_EQUAL">VRef ≥ VBuf</option>
    <option value="7" name="TEST_ALWAYS">Always true. Buffer is ignored.</option>
</enum>
*/

#[repr(u32)]
#[derive(Debug, NomLE)]

pub enum StencilTestFunc {
    Never = 0,
    Less = 1,
    Equal = 2,
    LessEqual = 3,
    Greater = 4,
    NotEqual = 5,
    GreaterEqual = 6,
    Always = 7,
}

/*
<enum name="StencilAction" storage="uint">
    Describes the actions which can occur as a result of tests for NiStencilProperty.
    <option value="0" name="ACTION_KEEP">Keep the current value in the stencil buffer.</option>
    <option value="1" name="ACTION_ZERO">Write zero to the stencil buffer.</option>
    <option value="2" name="ACTION_REPLACE">Write the reference value to the stencil buffer.</option>
    <option value="3" name="ACTION_INCREMENT">Increment the value in the stencil buffer.</option>
    <option value="4" name="ACTION_DECREMENT">Decrement the value in the stencil buffer.</option>
    <option value="5" name="ACTION_INVERT">Bitwise invert the value in the stencil buffer.</option>
</enum>
*/

#[repr(u32)]
#[derive(Debug, NomLE)]

pub enum StencilAction {
    Keep = 0,
    Zero = 1,
    Replace = 2,
    Increment = 3,
    Decrement = 4,
    Invert = 5,
}

/*
<enum name="StencilDrawMode" storage="uint">
    Describes the face culling options for NiStencilProperty.
    <option value="0" name="DRAW_CCW_OR_BOTH">Application default, chooses between DRAW_CCW or DRAW_BOTH.</option>
    <option value="1" name="DRAW_CCW">Draw only the triangles whose vertices are ordered CCW with respect to the viewer. (Standard behavior)</option>
    <option value="2" name="DRAW_CW">Draw only the triangles whose vertices are ordered CW with respect to the viewer. (Effectively flips faces)</option>
    <option value="3" name="DRAW_BOTH">Draw all triangles, regardless of orientation. (Effectively force double-sided)</option>
</enum>
*/

#[repr(u32)]
#[derive(Debug, NomLE)]

pub enum StencilDrawMode {
    CCWOrBoth = 0,
    CCW = 1,
    CW = 2,
    Both = 3,
}

/*
<enum name="TestFunction" storage="uint">
    Describes Z-buffer test modes for NiZBufferProperty.
    "Less than" = closer to camera, "Greater than" = further from camera.
    <option value="0" name="TEST_ALWAYS">Always true. Buffer is ignored.</option>
    <option value="1" name="TEST_LESS">VRef ‹ VBuf</option>
    <option value="2" name="TEST_EQUAL">VRef = VBuf</option>
    <option value="3" name="TEST_LESS_EQUAL">VRef ≤ VBuf</option>
    <option value="4" name="TEST_GREATER">VRef › VBuf</option>
    <option value="5" name="TEST_NOT_EQUAL">VRef ≠ VBuf</option>
    <option value="6" name="TEST_GREATER_EQUAL">VRef ≥ VBuf</option>
    <option value="7" name="TEST_NEVER">Always false. Ref value is ignored.</option>
</enum>
*/

#[repr(u32)]
#[derive(Debug, NomLE)]

pub enum TestFunction {
    Always = 0,
    Less = 1,
    Equal = 2,
    LessEqual = 3,
    Greater = 4,
    NotEqual = 5,
    GreaterEqual = 6,
    Never = 7,
}

/*
<enum name="AlphaFunction" storage="ushort" prefix="ALPHA">
    Describes alpha blend modes for NiAlphaProperty.
    <option value="0" name="ONE" />
    <option value="1" name="ZERO" />
    <option value="2" name="SRC_COLOR" />
    <option value="3" name="INV_SRC_COLOR" />
    <option value="4" name="DEST_COLOR" />
    <option value="5" name="INV_DEST_COLOR" />
    <option value="6" name="SRC_ALPHA" />
    <option value="7" name="INV_SRC_ALPHA" />
    <option value="8" name="DEST_ALPHA" />
    <option value="9" name="INV_DEST_ALPHA" />
    <option value="10" name="SRC_ALPHA_SATURATE" />
</enum>
*/

#[repr(u16)]
#[derive(Debug, NomLE)]

pub enum AlphaFunction {
    One = 0,
    Zero = 1,
    SrcColor = 2,
    InvSrcColor = 3,
    DestColor = 4,
    InvDestColor = 5,
    SrcAlpha = 6,
    InvSrcAlpha = 7,
    DestAlpha = 8,
    InvDestAlpha = 9,
    SrcAlphaSaturate = 10,
}

/*
<enum name="hkMotionType" storage="byte" versions="#BETHESDA#">
    hkpMotion::MotionType. Motion type of a rigid body determines what happens when it is simulated.
    <option value="0" name="MO_SYS_INVALID">Invalid</option>
    <option value="1" name="MO_SYS_DYNAMIC">A fully-simulated, movable rigid body. At construction time the engine checks the input inertia and selects MO_SYS_SPHERE_INERTIA or MO_SYS_BOX_INERTIA as appropriate.</option>
    <option value="2" name="MO_SYS_SPHERE_INERTIA">Simulation is performed using a sphere inertia tensor.</option>
    <option value="3" name="MO_SYS_SPHERE_STABILIZED">This is the same as MO_SYS_SPHERE_INERTIA, except that simulation of the rigid body is "softened".</option>
    <option value="4" name="MO_SYS_BOX_INERTIA">Simulation is performed using a box inertia tensor.</option>
    <option value="5" name="MO_SYS_BOX_STABILIZED">This is the same as MO_SYS_BOX_INERTIA, except that simulation of the rigid body is "softened".</option>
    <option value="6" name="MO_SYS_KEYFRAMED">Simulation is not performed as a normal rigid body. The keyframed rigid body has an infinite mass when viewed by the rest of the system. (used for creatures)</option>
    <option value="7" name="MO_SYS_FIXED">This motion type is used for the static elements of a game scene, e.g. the landscape. Faster than MO_SYS_KEYFRAMED at velocity 0. (used for weapons)</option>
    <option value="8" name="MO_SYS_THIN_BOX">A box inertia motion which is optimized for thin boxes and has less stability problems</option>
    <option value="9" name="MO_SYS_CHARACTER">A specialized motion used for character controllers</option>
</enum>
*/

#[repr(u8)]
#[derive(Debug, NomLE)]
pub enum HKMotionType {
    Invalid = 0,
    Dynamic = 1,
    SphereInertia = 2,
    SphereStabilized = 3,
    BoxInertia = 4,
    BoxStabilized = 5,
    Keyframed = 6,
    Fixed = 7,
    ThinBox = 8,
    Character = 9,
}

/*
<enum name="hkDeactivatorType" storage="byte" versions="#BETHESDA#">
    hkpRigidBodyDeactivator::DeactivatorType. Deactivator Type determines which mechanism Havok will use to classify the body as deactivated.
    <option value="0" name="DEACTIVATOR_INVALID">Invalid</option>
    <option value="1" name="DEACTIVATOR_NEVER">This will force the rigid body to never deactivate.</option>
    <option value="2" name="DEACTIVATOR_SPATIAL">Tells Havok to use a spatial deactivation scheme. This makes use of high and low frequencies of positional motion to determine when deactivation should occur.</option>
</enum>
*/

#[repr(u8)]
#[derive(Debug, NomLE)]
pub enum HKDeactivatorType {
    Invalid = 0,
    Never = 1,
    Spatial = 2,
}

/*
<enum name="hkSolverDeactivation" storage="byte" versions="#BETHESDA#">
    hkpRigidBodyCinfo::SolverDeactivation.
    A list of possible solver deactivation settings. This value defines how aggressively the solver deactivates objects.
    Note: Solver deactivation does not save CPU, but reduces creeping of movable objects in a pile quite dramatically.
    <option value="0" name="SOLVER_DEACTIVATION_INVALID">Invalid</option>
    <option value="1" name="SOLVER_DEACTIVATION_OFF">No solver deactivation.</option>
    <option value="2" name="SOLVER_DEACTIVATION_LOW">Very conservative deactivation, typically no visible artifacts.</option>
    <option value="3" name="SOLVER_DEACTIVATION_MEDIUM">Normal deactivation, no serious visible artifacts in most cases.</option>
    <option value="4" name="SOLVER_DEACTIVATION_HIGH">Fast deactivation, visible artifacts.</option>
    <option value="5" name="SOLVER_DEACTIVATION_MAX">Very fast deactivation, visible artifacts.</option>
</enum>
*/

#[repr(u8)]
#[derive(Debug, NomLE)]
pub enum HKSolverDeactivation {
    Invalid = 0,
    Off = 1,
    Low = 2,
    Medium = 3,
    High = 4,
    Max = 5,
}

/*
<enum name="hkQualityType" storage="byte" versions="#BETHESDA#">
    hkpCollidableQualityType. Describes the priority and quality of collisions for a body,
        e.g. you may expect critical game play objects to have solid high-priority collisions so that they never sink into ground,
        or may allow penetrations for visual debris objects.
    Notes:
        - Fixed and keyframed objects cannot interact with each other.
        - Debris can interpenetrate but still responds to Bullet hits.
        - Critical objects are forced to not interpenetrate.
        - Moving objects can interpenetrate slightly with other Moving or Debris objects but nothing else.
    <option value="0" name="MO_QUAL_INVALID">Automatically assigned to MO_QUAL_FIXED, MO_QUAL_KEYFRAMED or MO_QUAL_DEBRIS</option>
    <option value="1" name="MO_QUAL_FIXED">Static body.</option>
    <option value="2" name="MO_QUAL_KEYFRAMED">Animated body with infinite mass.</option>
    <option value="3" name="MO_QUAL_DEBRIS">Low importance bodies adding visual detail.</option>
    <option value="4" name="MO_QUAL_MOVING">Moving bodies which should not penetrate or leave the world, but can.</option>
    <option value="5" name="MO_QUAL_CRITICAL">Gameplay critical bodies which cannot penetrate or leave the world under any circumstance.</option>
    <option value="6" name="MO_QUAL_BULLET">Fast-moving bodies, such as projectiles.</option>
    <option value="7" name="MO_QUAL_USER">For user.</option>
    <option value="8" name="MO_QUAL_CHARACTER">For use with rigid body character controllers.</option>
    <option value="9" name="MO_QUAL_KEYFRAMED_REPORT">
        Moving bodies with infinite mass which should report contact points and TOI collisions against all other bodies.
    </option>
</enum>
*/

#[repr(u8)]
#[derive(Debug, NomLE)]
pub enum HKQualityType {
    Invalid = 0,
    Fixed = 1,
    Keyframed = 2,
    Debris = 3,
    Moving = 4,
    Critical = 5,
    Bullet = 6,
    User = 7,
    Character = 8,
    KeyframedReport = 9,
}

/*
<enum name="ForceType" storage="uint">
    Describes the type of gravitational force.
    <option value="0" name="FORCE_PLANAR"></option>
    <option value="1" name="FORCE_SPHERICAL"></option>
    <option value="2" name="FORCE_UNKNOWN"></option>
</enum>
*/

#[repr(u32)]
#[derive(Debug, NomLE)]
pub enum ForceType {
    Planar = 0,
    Spherical = 1,
    Unknown = 2,
}

/*
<enum name="TransformMember" storage="uint">
    Describes which aspect of the NiTextureTransform the NiTextureTransformController will modify.
    <option value="0" name="TT_TRANSLATE_U">Control the translation of the U coordinates.</option>
    <option value="1" name="TT_TRANSLATE_V">Control the translation of the V coordinates.</option>
    <option value="2" name="TT_ROTATE">Control the rotation of the coordinates.</option>
    <option value="3" name="TT_SCALE_U">Control the scale of the U coordinates.</option>
    <option value="4" name="TT_SCALE_V">Control the scale of the V coordinates.</option>
</enum>
*/

#[repr(u32)]
#[derive(Debug, NomLE)]

pub enum TransformMember {
    TranslateU = 0,
    TranslateV = 1,
    Rotate = 2,
    ScaleU = 3,
    ScaleV = 4,
}

/*
<enum name="DecayType" storage="uint">
    Describes the decay function of bomb forces.
    <option value="0" name="DECAY_NONE">No decay.</option>
    <option value="1" name="DECAY_LINEAR">Linear decay.</option>
    <option value="2" name="DECAY_EXPONENTIAL">Exponential decay.</option>
</enum>
*/

#[repr(u32)]
#[derive(Debug, NomLE)]

pub enum DecayType {
    None = 0,
    Linear = 1,
    Exponential = 2,
}

/*
<enum name="SymmetryType" storage="uint">
    Describes the symmetry type of bomb forces.
    <option value="0" name="SPHERICAL_SYMMETRY">Spherical Symmetry.</option>
    <option value="1" name="CYLINDRICAL_SYMMETRY">Cylindrical Symmetry.</option>
    <option value="2" name="PLANAR_SYMMETRY">Planar Symmetry.</option>
</enum>
*/

#[repr(u32)]
#[derive(Debug, NomLE)]

pub enum SymmetryType {
    Spherical = 0,
    Cylindrical = 1,
    Planar = 2,
}

/*
<enum name="VelocityType" storage="uint">
    Controls the way the a particle mesh emitter determines the starting speed and direction of the particles that are emitted.
    <option value="0" name="VELOCITY_USE_NORMALS">Uses the normals of the meshes to determine staring velocity.</option>
    <option value="1" name="VELOCITY_USE_RANDOM">Starts particles with a random velocity.</option>
    <option value="2" name="VELOCITY_USE_DIRECTION">Uses the emission axis to determine initial particle direction?</option>
</enum>
*/

#[repr(u32)]
#[derive(Debug, NomLE)]

pub enum VelocityType {
    UseNormals = 0,
    UseRandom = 1,
    UseDirection = 2,
}

/*
<enum name="EmitFrom" storage="uint">
    Controls which parts of the mesh that the particles are emitted from.
    <option value="0" name="EMIT_FROM_VERTICES">Emit particles from the vertices of the mesh.</option>
    <option value="1" name="EMIT_FROM_FACE_CENTER">Emit particles from the center of the faces of the mesh.</option>
    <option value="2" name="EMIT_FROM_EDGE_CENTER">Emit particles from the center of the edges of the mesh.</option>
    <option value="3" name="EMIT_FROM_FACE_SURFACE">Perhaps randomly emit particles from anywhere on the faces of the mesh?</option>
    <option value="4" name="EMIT_FROM_EDGE_SURFACE">Perhaps randomly emit particles from anywhere on the edges of the mesh?</option>
</enum>
*/

#[repr(u32)]
#[derive(Debug, NomLE)]

pub enum EmitFrom {
    Vertices = 0,
    FaceCenter = 1,
    EdgeCenter = 2,
    FaceSurface = 3,
    EdgeSurface = 4,
}

/*
<enum name="TextureType" storage="uint">
    The type of information that is stored in a texture used by an NiTextureEffect.
    <option value="0" name="TEX_PROJECTED_LIGHT">Apply a projected light texture. Each light effect is summed before multiplying by the base texture.</option>
    <option value="1" name="TEX_PROJECTED_SHADOW">Apply a projected shadow texture. Each shadow effect is multiplied by the base texture.</option>
    <option value="2" name="TEX_ENVIRONMENT_MAP">Apply an environment map texture. Added to the base texture and light/shadow/decal maps.</option>
    <option value="3" name="TEX_FOG_MAP">Apply a fog map texture. Alpha channel is used to blend the color channel with the base texture.</option>
</enum>
*/

#[repr(u32)]
#[derive(Debug, NomLE)]

pub enum TextureType {
    ProjectedLight = 0,
    ProjectedShadow = 1,
    EnvironmentMap = 2,
    FogMap = 3,
}

/*
<enum name="CoordGenType" storage="uint">
    Determines the way that UV texture coordinates are generated.
    <option value="0" name="CG_WORLD_PARALLEL">Use planar mapping.</option>
    <option value="1" name="CG_WORLD_PERSPECTIVE">Use perspective mapping.</option>
    <option value="2" name="CG_SPHERE_MAP">Use spherical mapping.</option>
    <option value="3" name="CG_SPECULAR_CUBE_MAP">Use specular cube mapping. For NiSourceCubeMap only.</option>
    <option value="4" name="CG_DIFFUSE_CUBE_MAP">Use diffuse cube mapping. For NiSourceCubeMap only.</option>
</enum>
*/

#[repr(u32)]
#[derive(Debug, NomLE)]

pub enum CoordGenType {
    WorldParallel = 0,
    WorldPerspective = 1,
    SphereMap = 2,
    SpecularCubeMap = 3,
    DiffuseCubeMap = 4,
}

/*
<enum name="EndianType" storage="byte">
    <option value="0" name="ENDIAN_BIG">The numbers are stored in big endian format, such as those used by PowerPC Mac processors.</option>
    <option value="1" name="ENDIAN_LITTLE">The numbers are stored in little endian format, such as those used by Intel and AMD x86 processors.</option>
</enum>
*/

#[repr(u8)]
#[derive(Debug, NomLE)]

pub enum EndianType {
    Big = 0,
    Little = 1,
}

/*
<enum name="MaterialColor" storage="ushort">
    Used by NiMaterialColorControllers to select which type of color in the controlled object that will be animated.
    <option value="0" name="TC_AMBIENT">Control the ambient color.</option>
    <option value="1" name="TC_DIFFUSE">Control the diffuse color.</option>
    <option value="2" name="TC_SPECULAR">Control the specular color.</option>
    <option value="3" name="TC_SELF_ILLUM">Control the self illumination color.</option>
</enum>
*/

#[repr(u16)]
#[derive(Debug, NomLE)]
pub enum MaterialColor {
    Ambient = 0,
    Diffuse = 1,
    Specular = 2,
    SelfIllum = 3,
}

/*
<enum name="LightColor" storage="ushort">
    Used by NiLightColorControllers to select which type of color in the controlled object that will be animated.
    <option value="0" name="LC_DIFFUSE">Control the diffuse color.</option>
    <option value="1" name="LC_AMBIENT">Control the ambient color.</option>
</enum>
*/

#[repr(u16)]
#[derive(Debug, NomLE)]
pub enum LightColor {
    Diffuse = 0,
    Ambient = 1,
}

/*
<enum name="ConsistencyType" storage="ushort">
    Used by NiGeometryData to control the volatility of the mesh.
    Consistency Type is masked to only the upper 4 bits (0xF000). Dirty mask is the lower 12 (0x0FFF) but only used at runtime.
    <option value="0x0000" name="CT_MUTABLE">Mutable Mesh</option>
    <option value="0x4000" name="CT_STATIC">Static Mesh</option>
    <option value="0x8000" name="CT_VOLATILE">Volatile Mesh</option>
</enum>
*/

#[repr(u16)]
#[derive(Debug, NomLE)]
pub enum ConsistencyType {
    Mutable = 0x0000,
    Static = 0x4000,
    Volatile = 0x8000,
}

/*
<enum name="SortingMode" storage="uint">
    Describes the way that NiSortAdjustNode modifies the sorting behavior for the subtree below it.
    <option value="0" name="SORTING_INHERIT">Inherit. Acts identical to NiNode.</option>
    <option value="1" name="SORTING_OFF">Disables sort on all geometry under this node.</option>
</enum>
*/

#[repr(u32)]
#[derive(Debug, NomLE)]

pub enum SortingMode {
    Inherit = 0,
    Off = 1,
}

/*
<enum name="PropagationMode" storage="uint">
    The propagation mode controls scene graph traversal during collision detection operations for NiCollisionData.
    <option value="0" name="PROPAGATE_ON_SUCCESS">Propagation only occurs as a result of a successful collision.</option>
    <option value="1" name="PROPAGATE_ON_FAILURE">(Deprecated) Propagation only occurs as a result of a failed collision.</option>
    <option value="2" name="PROPAGATE_ALWAYS">Propagation always occurs regardless of collision result.</option>
    <option value="3" name="PROPAGATE_NEVER">Propagation never occurs regardless of collision result.</option>
    <option value="6" name="PROPAGATE_UNKNOWN_6">Propagation mode found in Civ IV Chariot_Celtic.</option>
</enum>
*/

#[repr(u32)]
#[derive(Debug, NomLE)]

pub enum PropagationMode {
    OnSuccess = 0,
    OnFailure = 1,
    Always = 2,
    Never = 3,
    Unknown6 = 6,
}

/*
<enum name="CollisionMode" prefix="CM" storage="uint">
    The collision mode controls the type of collision operation that is to take place for NiCollisionData.
    <option value="0" name="USE_OBB">Use Bounding Box</option>
    <option value="1" name="USE_TRI">Use Triangles</option>
    <option value="2" name="USE_ABV">Use Alternate Bounding Volumes</option>
    <option value="3" name="NOTEST">Indicates that no collision test should be made.</option>
    <option value="4" name="USE_NIBOUND">Use NiBound</option>
</enum>
*/

#[repr(u32)]
#[derive(Debug, NomLE)]

pub enum CollisionMode {
    OBB = 0,
    Tri = 1,
    ABV = 2,
    NoTest = 3,
    NiBound = 4,
}

/*
<enum name="BoundVolumeType" storage="uint">
    <option value="0xffffffff" name="BASE_BV">Default</option>
    <option value="0" name="SPHERE_BV">Sphere</option>
    <option value="1" name="BOX_BV">Box</option>
    <option value="2" name="CAPSULE_BV">Capsule</option>
    <option value="4" name="UNION_BV">Union</option>
    <option value="5" name="HALFSPACE_BV">Half Space</option>
</enum>
*/

#[repr(u32)]
#[derive(Debug, NomLE)]
pub enum BoundVolumeType {
    Base = 0xffffffff,
    Sphere = 0,
    Box = 1,
    Capsule = 2,
    Union = 4,
    HalfSpace = 5,
}

/*
<enum name="hkResponseType" storage="byte" versions="#BETHESDA#">
    hkpMaterial::ResponseType
    <option value="0" name="RESPONSE_INVALID">Invalid Response</option>
    <option value="1" name="RESPONSE_SIMPLE_CONTACT">Do normal collision resolution</option>
    <option value="2" name="RESPONSE_REPORTING">No collision resolution is performed but listeners are called</option>
    <option value="3" name="RESPONSE_NONE">Do nothing, ignore all the results.</option>
</enum>
*/

#[repr(u8)]
#[derive(Debug, NomLE)]
pub enum HKResponseType {
    Invalid = 0,
    SimpleContact = 1,
    Reporting = 2,
    None = 3,
}

/*
<enum name="BSDismemberBodyPartType" storage="ushort" versions="#BETHESDA#">
    Biped bodypart data used for visibility control of triangles.  Options are Fallout 3, except where marked for Skyrim (uses SBP prefix)
    Skyrim BP names are listed only for vanilla names, different creatures have different defnitions for naming.
    <option value="0" name="BP_TORSO">Torso</option>
    <option value="1" name="BP_HEAD">Head</option>
    <option value="2" name="BP_HEAD2">Head 2</option>
    <option value="3" name="BP_LEFTARM">Left Arm</option>
    <option value="4" name="BP_LEFTARM2">Left Arm 2</option>
    <option value="5" name="BP_RIGHTARM">Right Arm</option>
    <option value="6" name="BP_RIGHTARM2">Right Arm 2</option>
    <option value="7" name="BP_LEFTLEG">Left Leg</option>
    <option value="8" name="BP_LEFTLEG2">Left Leg 2</option>
    <option value="9" name="BP_LEFTLEG3">Left Leg 3</option>
    <option value="10" name="BP_RIGHTLEG">Right Leg</option>
    <option value="11" name="BP_RIGHTLEG2">Right Leg 2</option>
    <option value="12" name="BP_RIGHTLEG3">Right Leg 3</option>
    <option value="13" name="BP_BRAIN">Brain</option>

    <option value="30" name="SBP_30_HEAD">Skyrim, Head(Human), Body(Atronachs,Beasts), Mask(Dragonpriest)</option>
    <option value="31" name="SBP_31_HAIR">Skyrim, Hair(human), Far(Dragon), Mask2(Dragonpriest),SkinnedFX(Spriggan)</option>
    <option value="32" name="SBP_32_BODY">Skyrim, Main body, extras(Spriggan)</option>
    <option value="33" name="SBP_33_HANDS">Skyrim, Hands L/R, BodyToo(Dragonpriest), Legs(Draugr), Arms(Giant)</option>
    <option value="34" name="SBP_34_FOREARMS">Skyrim, Forearms L/R, Beard(Draugr)</option>
    <option value="35" name="SBP_35_AMULET">Skyrim, Amulet</option>
    <option value="36" name="SBP_36_RING">Skyrim, Ring</option>
    <option value="37" name="SBP_37_FEET">Skyrim, Feet L/R</option>
    <option value="38" name="SBP_38_CALVES">Skyrim, Calves L/R</option>
    <option value="39" name="SBP_39_SHIELD">Skyrim, Shield</option>
    <option value="40" name="SBP_40_TAIL">Skyrim, Tail(Argonian/Khajiit), Skeleton01(Dragon), FX01(AtronachStorm),FXMist (Dragonpriest), Spit(Chaurus,Spider),SmokeFins(IceWraith)</option>
    <option value="41" name="SBP_41_LONGHAIR">Skyrim, Long Hair(Human), Skeleton02(Dragon),FXParticles(Dragonpriest)</option>
    <option value="42" name="SBP_42_CIRCLET">Skyrim, Circlet(Human, MouthFireEffect(Dragon)</option>
    <option value="43" name="SBP_43_EARS">Skyrim, Ears</option>
    <option value="44" name="SBP_44_DRAGON_BLOODHEAD_OR_MOD_MOUTH">Skyrim, Bloodied dragon head, or NPC face/mouth</option>
    <option value="45" name="SBP_45_DRAGON_BLOODWINGL_OR_MOD_NECK">Skyrim, Left Bloodied dragon wing, Saddle(Horse), or NPC cape, scarf, shawl, neck-tie, etc.</option>
    <option value="46" name="SBP_46_DRAGON_BLOODWINGR_OR_MOD_CHEST_PRIMARY">Skyrim, Right Bloodied dragon wing, or NPC chest primary or outergarment</option>
    <option value="47" name="SBP_47_DRAGON_BLOODTAIL_OR_MOD_BACK">Skyrim, Bloodied dragon tail, or NPC backpack/wings/...</option>
    <option value="48" name="SBP_48_MOD_MISC1">Anything that does not fit in the list</option>
    <option value="49" name="SBP_49_MOD_PELVIS_PRIMARY">Pelvis primary or outergarment</option>
    <option value="50" name="SBP_50_DECAPITATEDHEAD">Skyrim, Decapitated Head</option>
    <option value="51" name="SBP_51_DECAPITATE">Skyrim, Decapitate, neck gore</option>
    <option value="52" name="SBP_52_MOD_PELVIS_SECONDARY">Pelvis secondary or undergarment</option>
    <option value="53" name="SBP_53_MOD_LEG_RIGHT">Leg primary or outergarment or right leg</option>
    <option value="54" name="SBP_54_MOD_LEG_LEFT">Leg secondary or undergarment or left leg</option>
    <option value="55" name="SBP_55_MOD_FACE_JEWELRY">Face alternate or jewelry</option>
    <option value="56" name="SBP_56_MOD_CHEST_SECONDARY">Chest secondary or undergarment</option>
    <option value="57" name="SBP_57_MOD_SHOULDER">Shoulder</option>
    <option value="58" name="SBP_58_MOD_ARM_LEFT">Arm secondary or undergarment or left arm</option>
    <option value="59" name="SBP_59_MOD_ARM_RIGHT">Arm primary or outergarment or right arm</option>
    <option value="60" name="SBP_60_MOD_MISC2">Anything that does not fit in the list</option>
    <option value="61" name="SBP_61_FX01">Skyrim, FX01(Humanoid)</option>

    <option value="101" name="BP_SECTIONCAP_HEAD">Section Cap | Head</option>
    <option value="102" name="BP_SECTIONCAP_HEAD2">Section Cap | Head 2</option>
    <option value="103" name="BP_SECTIONCAP_LEFTARM">Section Cap | Left Arm</option>
    <option value="104" name="BP_SECTIONCAP_LEFTARM2">Section Cap | Left Arm 2</option>
    <option value="105" name="BP_SECTIONCAP_RIGHTARM">Section Cap | Right Arm</option>
    <option value="106" name="BP_SECTIONCAP_RIGHTARM2">Section Cap | Right Arm 2</option>
    <option value="107" name="BP_SECTIONCAP_LEFTLEG">Section Cap | Left Leg</option>
    <option value="108" name="BP_SECTIONCAP_LEFTLEG2">Section Cap | Left Leg 2</option>
    <option value="109" name="BP_SECTIONCAP_LEFTLEG3">Section Cap | Left Leg 3</option>
    <option value="110" name="BP_SECTIONCAP_RIGHTLEG">Section Cap | Right Leg</option>
    <option value="111" name="BP_SECTIONCAP_RIGHTLEG2">Section Cap | Right Leg 2</option>
    <option value="112" name="BP_SECTIONCAP_RIGHTLEG3">Section Cap | Right Leg 3</option>
    <option value="113" name="BP_SECTIONCAP_BRAIN">Section Cap | Brain</option>

    <option value="130" name="SBP_130_HEAD">Skyrim, Head slot, use on full-face helmets</option>
    <option value="131" name="SBP_131_HAIR">Skyrim, Hair slot 1, use on hoods</option>
    <option value="132" name="SBP_132_HAIR">Skyrim, Hair slot 2?, use on hoods</option>
    <option value="141" name="SBP_141_LONGHAIR">Skyrim, Hair slot 2, use for longer hair</option>
    <option value="142" name="SBP_142_CIRCLET">Skyrim, Circlet slot 1, use for circlets</option>
    <option value="143" name="SBP_143_EARS">Skyrim, Ear slot</option>
    <option value="150" name="SBP_150_DECAPITATEDHEAD">Skyrim, neck gore on head side</option>

    <option value="201" name="BP_TORSOCAP_HEAD">Torso Cap | Head</option>
    <option value="202" name="BP_TORSOCAP_HEAD2">Torso Cap | Head 2</option>
    <option value="203" name="BP_TORSOCAP_LEFTARM">Torso Cap | Left Arm</option>
    <option value="204" name="BP_TORSOCAP_LEFTARM2">Torso Cap | Left Arm 2</option>
    <option value="205" name="BP_TORSOCAP_RIGHTARM">Torso Cap | Right Arm</option>
    <option value="206" name="BP_TORSOCAP_RIGHTARM2">Torso Cap | Right Arm 2</option>
    <option value="207" name="BP_TORSOCAP_LEFTLEG">Torso Cap | Left Leg</option>
    <option value="208" name="BP_TORSOCAP_LEFTLEG2">Torso Cap | Left Leg 2</option>
    <option value="209" name="BP_TORSOCAP_LEFTLEG3">Torso Cap | Left Leg 3</option>
    <option value="210" name="BP_TORSOCAP_RIGHTLEG">Torso Cap | Right Leg</option>
    <option value="211" name="BP_TORSOCAP_RIGHTLEG2">Torso Cap | Right Leg 2</option>
    <option value="212" name="BP_TORSOCAP_RIGHTLEG3">Torso Cap | Right Leg 3</option>
    <option value="213" name="BP_TORSOCAP_BRAIN">Torso Cap | Brain</option>

    <option value="230" name="SBP_230_HEAD">Skyrim, Head slot, use for neck on character head</option>

    <option value="1000" name="BP_TORSOSECTION_HEAD">Torso Section | Head</option>
    <option value="2000" name="BP_TORSOSECTION_HEAD2">Torso Section | Head 2</option>
    <option value="3000" name="BP_TORSOSECTION_LEFTARM">Torso Section | Left Arm</option>
    <option value="4000" name="BP_TORSOSECTION_LEFTARM2">Torso Section | Left Arm 2</option>
    <option value="5000" name="BP_TORSOSECTION_RIGHTARM">Torso Section | Right Arm</option>
    <option value="6000" name="BP_TORSOSECTION_RIGHTARM2">Torso Section | Right Arm 2</option>
    <option value="7000" name="BP_TORSOSECTION_LEFTLEG">Torso Section | Left Leg</option>
    <option value="8000" name="BP_TORSOSECTION_LEFTLEG2">Torso Section | Left Leg 2</option>
    <option value="9000" name="BP_TORSOSECTION_LEFTLEG3">Torso Section | Left Leg 3</option>
    <option value="10000" name="BP_TORSOSECTION_RIGHTLEG">Torso Section | Right Leg</option>
    <option value="11000" name="BP_TORSOSECTION_RIGHTLEG2">Torso Section | Right Leg 2</option>
    <option value="12000" name="BP_TORSOSECTION_RIGHTLEG3">Torso Section | Right Leg 3</option>
    <option value="13000" name="BP_TORSOSECTION_BRAIN">Torso Section | Brain</option>
</enum>
*/

#[repr(u16)]
#[derive(Debug, NomLE)]
pub enum BSDismemberBodyPartType {
    Torso = 0,
    Head = 1,
    Head2 = 2,
    LeftArm = 3,
    LeftArm2 = 4,
    RightArm = 5,
    RightArm2 = 6,
    LeftLeg = 7,
    LeftLeg2 = 8,
    LeftLeg3 = 9,
    RightLeg = 10,
    RightLeg2 = 11,
    RightLeg3 = 12,
    Brain = 13,
    SBP30Head = 30,
    SBP31Hair = 31,
    SBP32Body = 32,
    SBP33Hands = 33,
    SBP34Forearms = 34,
    SBP35Amulet = 35,
    SBP36Ring = 36,
    SBP37Feet = 37,
    SBP38Calves = 38,
    SBP39Shield = 39,
    SBP40Tail = 40,
    SBP41LongHair = 41,
    SBP42Circlet = 42,
    SBP43Ears = 43,
    SBP44DragonBloodHeadOrModMouth = 44,
    SBP45DragonBloodWingLOrModNeck = 45,
    SBP46DragonBloodWingROrModChestPrimary = 46,
    SBP47DragonBloodTailOrModBack = 47,
    SBP48ModMisc1 = 48,
    SBP49ModPelvisPrimary = 49,
    SBP50DecapitatedHead = 50,
    SBP51Decapitate = 51,
    SBP52ModPelvisSecondary = 52,
    SBP53ModLegRight = 53,
    SBP54ModLegLeft = 54,
    SBP55ModFaceJewelry = 55,
    SBP56ModChestSecondary = 56,
    SBP57ModShoulder = 57,
    SBP58ModArmLeft = 58,
    SBP59ModArmRight = 59,
    SBP60ModMisc2 = 60,
    SBP61FX01 = 61,
    BPSectionCapHead = 101,
    BPSectionCapHead2 = 102,
    BPSectionCapLeftArm = 103,
    BPSectionCapLeftArm2 = 104,
    BPSectionCapRightArm = 105,
    BPSectionCapRightArm2 = 106,
    BPSectionCapLeftLeg = 107,
    BPSectionCapLeftLeg2 = 108,
    BPSectionCapLeftLeg3 = 109,
    BPSectionCapRightLeg = 110,
    BPSectionCapRightLeg2 = 111,
    BPSectionCapRightLeg3 = 112,
    BPSectionCapBrain = 113,
    SBP130Head = 130,
    SBP131Hair = 131,
    SBP132Hair = 132,
    SBP141LongHair = 141,
    SBP142Circlet = 142,
    SBP143Ears = 143,
    SBP150DecapitatedHead = 150,
    BPTorsoCapHead = 201,
    BPTorsoCapHead2 = 202,
    BPTorsoCapLeftArm = 203,
    BPTorsoCapLeftArm2 = 204,
    BPTorsoCapRightArm = 205,
    BPTorsoCapRightArm2 = 206,
    BPTorsoCapLeftLeg = 207,
    BPTorsoCapLeftLeg2 = 208,
    BPTorsoCapLeftLeg3 = 209,
    BPTorsoCapRightLeg = 210,
    BPTorsoCapRightLeg2 = 211,
    BPTorsoCapRightLeg3 = 212,
    BPTorsoCapBrain = 213,
    SBP230Head = 230,
    BPTorsoSectionHead = 1000,
    BPTorsoSectionHead2 = 2000,
    BPTorsoSectionLeftArm = 3000,
    BPTorsoSectionLeftArm2 = 4000,
    BPTorsoSectionRightArm = 5000,
    BPTorsoSectionRightArm2 = 6000,
    BPTorsoSectionLeftLeg = 7000,
    BPTorsoSectionLeftLeg2 = 8000,
    BPTorsoSectionLeftLeg3 = 9000,
    BPTorsoSectionRightLeg = 10000,
    BPTorsoSectionRightLeg2 = 11000,
    BPTorsoSectionRightLeg3 = 12000,
    BPTorsoSectionBrain = 13000,
}

/*
<enum name="BSLightingShaderType" storage="uint" prefix="ST" versions="#SKY_AND_LATER#">
    Values for configuring the shader type in a BSLightingShaderProperty
    <option value="0" name="Default" />
    <option value="1" name="Environment Map">Enables EnvMap Mask(TS6), EnvMap Scale</option>
    <option value="2" name="Glow Shader">Enables Glow(TS3)</option>
    <option value="3" name="Parallax">Enables Height(TS4)</option>
    <option value="4" name="Face Tint">Enables Detail(TS4), Tint(TS7)</option>
    <option value="5" name="Skin Tint">Enables Skin Tint Color</option>
    <option value="6" name="Hair Tint">Enables Hair Tint Color</option>
    <option value="7" name="Parallax Occ">Enables Height(TS4), Max Passes, Scale. Unimplemented.</option>
    <option value="8" name="Multitexture Landscape" />
    <option value="9" name="LOD Landscape" />
    <option value="10" name="Snow" />
    <option value="11" name="MultiLayer Parallax">Enables EnvMap Mask(TS6), Layer(TS7), Parallax Layer Thickness, Parallax Refraction Scale, Parallax Inner Layer U Scale, Parallax Inner Layer V Scale, EnvMap Scale</option>
    <option value="12" name="Tree Anim" />
    <option value="13" name="LOD Objects" />
    <option value="14" name="Sparkle Snow">Enables SparkleParams</option>
    <option value="15" name="LOD Objects HD" />
    <option value="16" name="Eye Envmap">Enables EnvMap Mask(TS6), Eye EnvMap Scale</option>
    <option value="17" name="Cloud" />
    <option value="18" name="LOD Landscape Noise" />
    <option value="19" name="Multitexture Landscape LOD Blend" />
    <option value="20" name="FO4 Dismemberment" />
</enum>
*/

#[repr(u32)]
#[derive(Debug, NomLE)]

pub enum BSLightingShaderType {
    Default = 0,
    EnvironmentMap = 1,
    GlowShader = 2,
    Parallax = 3,
    FaceTint = 4,
    SkinTint = 5,
    HairTint = 6,
    ParallaxOcc = 7,
    MultitextureLandscape = 8,
    LODLandscape = 9,
    Snow = 10,
    MultiLayerParallax = 11,
    TreeAnim = 12,
    LODObjects = 13,
    SparkleSnow = 14,
    LODObjectsHD = 15,
    EyeEnvmap = 16,
    Cloud = 17,
    LODLandscapeNoise = 18,
    MultitextureLandscapeLODBlend = 19,
    FO4Dismemberment = 20,
}

/*
<enum name="BSShaderType155" storage="uint" prefix="ST155" versions="#F76#">
    Values for configuring the shader type in a BSLightingShaderProperty
    <option value="0" name="Default" />
    <option value="2" name="Glow" />
    <option value="3" name="Face Tint" />
    <option value="4" name="Skin Tint" />
    <option value="5" name="Hair Tint" />
    <option value="12" name="Eye Envmap">Enables EnvMap Mask, Eye EnvMap Scale</option>
    <option value="17" name="Terrain" />
</enum>
*/

#[repr(u32)]
#[derive(Debug, NomLE)]

pub enum BSShaderType155 {
    Default = 0,
    Glow = 2,
    FaceTint = 3,
    SkinTint = 4,
    HairTint = 5,
    EyeEnvmap = 12,
    Terrain = 17,
}

/*
<enum name="EffectShaderControlledVariable" storage="uint" prefix="ESCV" versions="#SKY_AND_LATER#">
    An unsigned 32-bit integer, describing which float variable in BSEffectShaderProperty to animate.
    <option value="0" name="EmissiveMultiple">EmissiveMultiple.</option>
    <option value="1" name="Falloff Start Angle">Falloff Start Angle (degrees).</option>
    <option value="2" name="Falloff Stop Angle">Falloff Stop Angle (degrees).</option>
    <option value="3" name="Falloff Start Opacity">Falloff Start Opacity.</option>
    <option value="4" name="Falloff Stop Opacity">Falloff Stop Opacity.</option>
    <option value="5" name="Alpha Transparency">Alpha Transparency (Emissive alpha?).</option>
    <option value="6" name="U Offset">U Offset.</option>
    <option value="7" name="U Scale">U Scale.</option>
    <option value="8" name="V Offset">V Offset.</option>
    <option value="9" name="V Scale">V Scale.</option>
    <option value="11" name="Unknown 11" />
    <option value="12" name="Unknown 12" />
    <option value="13" name="Unknown 13" />
    <option value="14" name="Unknown 14" />
</enum>
*/

#[repr(u32)]
#[derive(Debug, NomLE)]
pub enum EffectShaderControlledVariable {
    EmissiveMultiple = 0,
    FalloffStartAngle = 1,
    FalloffStopAngle = 2,
    FalloffStartOpacity = 3,
    FalloffStopOpacity = 4,
    AlphaTransparency = 5,
    UOffset = 6,
    UScale = 7,
    VOffset = 8,
    VScale = 9,
    Unknown11 = 11,
    Unknown12 = 12,
    Unknown13 = 13,
    Unknown14 = 14,
}

/*
<enum name="EffectShaderControlledColor" storage="uint" prefix="ECSC" versions="#SKY_AND_LATER#">
    An unsigned 32-bit integer, describing which color in BSEffectShaderProperty to animate.
    <option value="0" name="Emissive Color">Emissive Color.</option>
</enum>
*/

#[repr(u32)]
#[derive(Debug, NomLE)]

pub enum EffectShaderControlledColor {
    EmissiveColor = 0,
}

/*
<enum name="LightingShaderControlledFloat" storage="uint" prefix="LSCF" versions="#SKY_AND_LATER#">
    An unsigned 32-bit integer, describing which float variable in BSLightingShaderProperty to animate.
    <option value="0" name="Refraction Strength">The amount of distortion.</option>
    <option value="3" name="Unknown 3" />
    <option value="4" name="Unknown 4" />
    <option value="8" name="Environment Map Scale">Environment Map Scale.</option>
    <option value="9" name="Glossiness">Glossiness.</option>
    <option value="10" name="Specular Strength">Specular Strength.</option>
    <option value="11" name="Emissive Multiple">Emissive Multiple.</option>
    <option value="12" name="Alpha">Alpha.</option>
    <option value="13" name="Unknown 13" />
    <option value="14" name="Unknown 14" />
    <option value="20" name="U Offset">U Offset.</option>
    <option value="21" name="U Scale">U Scale.</option>
    <option value="22" name="V Offset">V Offset.</option>
    <option value="23" name="V Scale">V Scale.</option>
</enum>
*/

#[repr(u32)]
#[derive(Debug, NomLE)]

pub enum LightingShaderControlledFloat {
    RefractionStrength = 0,
    Unknown3 = 3,
    Unknown4 = 4,
    EnvironmentMapScale = 8,
    Glossiness = 9,
    SpecularStrength = 10,
    EmissiveMultiple = 11,
    Alpha = 12,
    Unknown13 = 13,
    Unknown14 = 14,
    UOffset = 20,
    UScale = 21,
    VOffset = 22,
    VScale = 23,
}

/*
<enum name="LightingShaderControlledUShort" storage="uint" prefix="LSCU" versions="#SKY_AND_LATER#">
    An unsigned 32-bit integer, describing which integral value in BSLightingShaderProperty to animate.
    <option value="0" name="Unknown 1" />
    <option value="1" name="Unknown 2" />
</enum>
*/

#[repr(u32)]
#[derive(Debug, NomLE)]

pub enum LightingShaderControlledUShort {
    Unknown1 = 0,
    Unknown2 = 1,
}

/*
<enum name="LightingShaderControlledColor" storage="uint" prefix="LSCC" versions="#SKY_AND_LATER#">
    An unsigned 32-bit integer, describing which color in BSLightingShaderProperty to animate.
    <option value="0" name="Specular Color">Specular Color.</option>
    <option value="1" name="Emissive Color">Emissive Color.</option>
</enum>
*/

#[repr(u32)]
#[derive(Debug, NomLE)]

pub enum LightingShaderControlledColor {
    SpecularColor = 0,
    EmissiveColor = 1,
}

/*
<enum name="hkConstraintType" storage="uint" versions="#BETHESDA#">
    hkpConstraintData::ConstraintType. Describes the type of bhkConstraint.
    <option value="0" name="BallAndSocket">A ball and socket constraint.</option>
    <option value="1" name="Hinge">A hinge constraint.</option>
    <option value="2" name="Limited Hinge">A limited hinge constraint.</option>
    <option value="6" name="Prismatic">A prismatic constraint.</option>
    <option value="7" name="Ragdoll">A ragdoll constraint.</option>
    <option value="8" name="StiffSpring">A stiff spring constraint.</option>
    <option value="13" name="Malleable">A malleable constraint.</option>
</enum>
*/

#[repr(u32)]
#[derive(Debug, NomLE)]
pub enum HKConstraintType {
    BallAndSocket = 0,
    Hinge = 1,
    LimitedHinge = 2,
    Prismatic = 6,
    Ragdoll = 7,
    StiffSpring = 8,
    Malleable = 13,
}

/*
<enum name="FogFunction" storage="ushort">
    <option value="0" name="FOG_Z_LINEAR" />
    <option value="1" name="FOG_RANGE_SQ" />
    <option value="2" name="FOG_VERTEX_ALPHA" />
</enum>
*/

#[repr(u16)]
#[derive(Debug, NomLE)]
pub enum FogFunction {
    ZLinear = 0,
    RangeSq = 1,
    VertexAlpha = 2,
}

/*
<enum name="AnimType" storage="ushort">
    <option value="0" name="APP_TIME" />
    <option value="1" name="APP_INIT" />
</enum>
*/

#[repr(u16)]
#[derive(Debug, NomLE)]
pub enum AnimType {
    Time = 0,
    Init = 1,
}

/*
<enum name="DitherFlags" storage="ushort">
    Flags for NiDitherProperty
    <option value="0" name="DITHER_DISABLED" />
    <option value="1" name="DITHER_ENABLED" />
</enum>
*/

#[repr(u16)]
#[derive(Debug, NomLE)]
pub enum DitherFlags {
    Disabled = 0,
    Enabled = 1,
}

/*
<enum name="ShadeFlags" storage="ushort">
    Flags for NiShadeProperty
    <option value="0" name="SHADING_HARD" />
    <option value="1" name="SHADING_SMOOTH" />
</enum>
*/

#[repr(u16)]
#[derive(Debug, NomLE)]
pub enum ShadeFlags {
    Hard = 0,
    Smooth = 1,
}

/*
<enum name="SpecularFlags" storage="ushort">
    Flags for NiSpecularProperty
    <option value="0" name="SPECULAR_DISABLED" />
    <option value="1" name="SPECULAR_ENABLED" />
</enum>
*/

#[repr(u16)]
#[derive(Debug, NomLE)]
pub enum SpecularFlags {
    Disabled = 0,
    Enabled = 1,
}

/*
<enum name="WireframeFlags" storage="ushort">
    Flags for NiWireframeProperty
    <option value="0" name="WIREFRAME_DISABLED" />
    <option value="1" name="WIREFRAME_ENABLED" />
</enum>
*/

#[repr(u16)]
#[derive(Debug, NomLE)]

pub enum WireframeFlags {
    Disabled = 0,
    Enabled = 1,
}

/*
<enum name="GeomMorpherFlags" storage="ushort">
    Flags for NiGeomMorpherController
    <option value="0" name="UPDATE_NORMALS_DISABLED" />
    <option value="1" name="UPDATE_NORMALS_ENABLED" />
</enum>
*/

#[repr(u16)]
#[derive(Debug, NomLE)]

pub enum GeomMorpherFlags {
    Disabled = 0,
    Enabled = 1,
}

/*
<enum name="AnimationType" storage="ushort" versions="#BETHESDA#">
    Bethesda Animation. Animation type used on this position. This specifies the function of this position.
    <option value="1" name="Sit">Actor use sit animation.</option>
    <option value="2" name="Sleep">Actor use sleep animation.</option>
    <option value="4" name="Lean">Used for lean animations?</option>
</enum>
*/

#[repr(u16)]
#[derive(Debug, NomLE)]
pub enum AnimationType {
    Sit = 1,
    Sleep = 2,
    Lean = 4,
}

/*
<enum name="BSCPCullingType" storage="uint" prefix="BSCP" versions="#FO3_AND_LATER#">
    Culling modes for multi bound nodes.
    <option value="0" name="CULL_NORMAL">Normal</option>
    <option value="1" name="CULL_ALLPASS">All Pass</option>
    <option value="2" name="CULL_ALLFAIL">All Fail</option>
    <option value="3" name="CULL_IGNOREMULTIBOUNDS">Ignore Multi Bounds</option>
    <option value="4" name="CULL_FORCEMULTIBOUNDSNOUPDATE">Force Multi Bounds No Update</option>
</enum>
*/

#[repr(u32)]
#[derive(Debug, NomLE)]
pub enum BSCPCullingType {
    Normal = 0,
    AllPass = 1,
    AllFail = 2,
    IgnoreMultiBounds = 3,
    ForceMultiBoundsNoUpdate = 4,
}

/*
<enum name="NiPSysModifierOrder" storage="uint">
    The set order for each derived class of NiPSysModifier.
    Note: For Skyrim, BSPSysStripUpdateModifier is 8000 and for FO3 it is 2500.
    <option value="0" name="ORDER_KILLOLDPARTICLES" />
    <option value="1" name="ORDER_BSLOD" />
    <option value="1000" name="ORDER_EMITTER" />
    <option value="2000" name="ORDER_SPAWN" />
    <option value="2500" name="ORDER_FO3_BSSTRIPUPDATE" />
    <option value="3000" name="ORDER_GENERAL" />
    <option value="4000" name="ORDER_FORCE" />
    <option value="5000" name="ORDER_COLLIDER" />
    <option value="6000" name="ORDER_POS_UPDATE" />
    <option value="6500" name="ORDER_POSTPOS_UPDATE" />
    <option value="6600" name="ORDER_WORLDSHIFT_PARTSPAWN" />
    <option value="7000" name="ORDER_BOUND_UPDATE" />
    <option value="8000" name="ORDER_SK_BSSTRIPUPDATE" />
</enum>
*/

#[repr(u32)]
#[derive(Debug, NomLE)]
pub enum NiPSysModifierOrder {
    KillOldParticles = 0,
    BSLOD = 1,
    Emitter = 1000,
    Spawn = 2000,
    FO3BSStripUpdate = 2500,
    General = 3000,
    Force = 4000,
    Collider = 5000,
    PosUpdate = 6000,
    PostPosUpdate = 6500,
    WorldShiftPartSpawn = 6600,
    BoundUpdate = 7000,
    SKBSStripUpdate = 8000,
}

/*
<enum name="SkyObjectType" storage="uint" versions="#BETHESDA#">
    Sets what sky function this object fulfills in BSSkyShaderProperty or SkyShaderProperty.
    <option value="0" name="BSSM_SKY_TEXTURE">BSSM_Sky_Texture</option>
    <option value="1" name="BSSM_SKY_SUNGLARE">BSSM_Sky_Sunglare</option>
    <option value="2" name="BSSM_SKY">BSSM_Sky</option>
    <option value="3" name="BSSM_SKY_CLOUDS">BSSM_Sky_Clouds</option>
    <option value="5" name="BSSM_SKY_STARS">BSSM_Sky_Stars</option>
    <option value="7" name="BSSM_SKY_MOON_STARS_MASK">BSSM_Sky_Moon_Stars_Mask</option>
</enum>
*/

#[repr(u32)]
#[derive(Debug, NomLE)]
pub enum SkyObjectType {
    BSSMSkyTexture = 0,
    BSSMSkySunglare = 1,
    BSSMSky = 2,
    BSSMSkyClouds = 3,
    BSSMSkyStars = 5,
    BSSMSkyMoonStarsMask = 7,
}
