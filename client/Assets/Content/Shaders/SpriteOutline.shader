Shader "Custom/SpriteOutline"
{
Properties
{
_MainTex ("Sprite Texture", 2D) = "white" {}
_OutlineColor ("Outline Color", Color) = (0,0,0,1)
_OutlineWidth ("Outline Width", Range(0,50)) = 2
_AlphaThreshold ("Alpha Threshold", Range(0,1)) = 0.1
}
SubShader
{
Tags { "Queue"="Transparent" "RenderType"="Transparent" "IgnoreProjector"="True" "CanUseSpriteAtlas"="True" }
Cull Off
Lighting Off
ZWrite Off
Blend One OneMinusSrcAlpha
Pass
{
CGPROGRAM
#pragma vertex vert
#pragma fragment frag
#include "UnityCG.cginc"

sampler2D _MainTex;
float4 _MainTex_TexelSize;
fixed4 _OutlineColor;
float _OutlineWidth;
float _AlphaThreshold;

struct appdata
{
float4 vertex : POSITION;
float2 uv : TEXCOORD0;
fixed4 color : COLOR;
};

struct v2f
{
float4 vertex : SV_POSITION;
float2 uv : TEXCOORD0;
fixed4 color : COLOR;
};

v2f vert(appdata v)
{
v2f o;
o.vertex = UnityObjectToClipPos(v.vertex);
o.uv = v.uv;
o.color = v.color;
return o;
}

fixed4 frag(v2f i) : SV_Target
{
fixed4 c = tex2D(_MainTex, i.uv) * i.color;
float a = c.a;
clip(_AlphaThreshold - a);
float2 texel = _MainTex_TexelSize.xy;
float maxA = 0.0;
int steps = (int)ceil(_OutlineWidth);
for (int s = 1; s <= steps; s++)
{
float scale = min((float)s, _OutlineWidth);
float2 d = texel * scale;
maxA = max(maxA, tex2D(_MainTex, i.uv + float2(d.x, 0)).a * i.color.a);
maxA = max(maxA, tex2D(_MainTex, i.uv - float2(d.x, 0)).a * i.color.a);
maxA = max(maxA, tex2D(_MainTex, i.uv + float2(0, d.y)).a * i.color.a);
maxA = max(maxA, tex2D(_MainTex, i.uv - float2(0, d.y)).a * i.color.a);
maxA = max(maxA, tex2D(_MainTex, i.uv + float2(d.x, d.y)).a * i.color.a);
maxA = max(maxA, tex2D(_MainTex, i.uv + float2(-d.x, d.y)).a * i.color.a);
maxA = max(maxA, tex2D(_MainTex, i.uv + float2(d.x, -d.y)).a * i.color.a);
maxA = max(maxA, tex2D(_MainTex, i.uv + float2(-d.x, -d.y)).a * i.color.a);
}
if (maxA > _AlphaThreshold)
{
fixed4 oc = _OutlineColor;
oc.a *= i.color.a;
oc.rgb *= oc.a;
return oc;
}
return fixed4(0,0,0,0);
}
ENDCG
}
}
}
