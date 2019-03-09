struct VsOutput {
    float4 gl_Position: SV_Position;
    float2 v_TexCoord: TEXCOORD;
    float4 v_Color: COLOR;
};

cbuffer Locals {
    float4x4  u_Proj;
    float2 u_Screen_Size;
};

VsOutput Vertex(float2 pos: a_Pos, float2 tc: a_TexCoord, float3 world_pos: a_World_Pos,
                int screen_rel: a_Screen_Rel, float4 color: a_Color) {


        // On-screen offset from text origin. 
        float2 v_Screen_Offset = float2(
            2 * pos.x / u_Screen_Size.x - 1,
            1 - 2 * pos.y / u_Screen_Size.y
        );
        float4 v_Screen_Pos = mul(u_Proj, world_pos);
        screen_rel = 0;  
        float2 v_World_Offset = screen_rel == 0
            // Perspective divide to get normalized device coords.

            ? float2 (
                v_Screen_Pos.x * 500,// / v_Screen_Pos.z + 1,
                v_Screen_Pos.y// / v_Screen_Pos.z - 1
            ) : float2(0.0, 0.0);

    VsOutput output = {
        float4(v_World_Offset + v_Screen_Offset, 0.0, 1.0),
        tc,
    	color,
    };
    return output;
}

Texture2D<float> t_Color;
SamplerState t_Color_; 

float4 Pixel(VsOutput pin) : SV_Target {
	// return float4(1,1,0,1);
    // return float4(pin.v_TexCoord.x, pin.v_TexCoord.y, 0, 1);
        float4 t_Font_Color = t_Color.Sample(t_Color_, pin.v_TexCoord);
        // return float4(pin.v_Color.a, pin.v_Color.a, pin.v_Color.a, 1);
        // return float4(t_Font_Color.a, t_Font_Color.a, t_Font_Color.a, 1);
        return float4(pin.v_Color.rgb, t_Font_Color.r * pin.v_Color.a);
}


//     #version 150 core

//     in vec4 v_Color;
//     in vec2 v_TexCoord;
//     out vec4 o_Color;
//     uniform sampler2D t_Color;

//     void main() {
//     }