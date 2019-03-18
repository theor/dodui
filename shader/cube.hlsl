struct VsOutput {
    float4 pos: SV_Position;
    float2 tc: TEXCOORD;
};

cbuffer Locals {
	float4x4 u_Transform;
    float4 u_Color;
	float2 u_Size;
};

float2 u_Screen;

VsOutput Vertex(float4 pos: a_Pos, float2 tc: a_TexCoord) {
    pos.xy = pos.xy * u_Size;
    VsOutput output = {
    	mul(u_Transform, pos),
    	tc,
    };
    return output;
}

Texture2D<float4> t_Color;
SamplerState t_Color_;

float4 Pixel(VsOutput pin) : SV_Target {
	float4 tex = u_Color;// float4(1.0,0,0,1);// t_Color.Sample(t_Color_, pin.tc);
	// float4 tex = t_Color.Sample(t_Color_, pin.tc) * u_Color;
    float blend = dot(pin.tc-0.5, pin.tc-0.5);
    return lerp(tex, 0.0, blend*1.0);   
}
