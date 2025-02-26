#nullable enable

using Dreamcaller.Components;
using Dreamcaller.Schema;
using Dreamcaller.Utils;
using UnityEngine;
using UnityEngine.AddressableAssets;

namespace Dreamcaller.Services
{
  public class AssetService : Service
  {
    public Sprite GetSprite(SpriteAddress address) => GetAsset<Sprite>(address.Sprite);

    public Font GetFont(FontAddress address) => GetAsset<Font>(address.Font);

    public AudioClip GetAudioClip(AudioClipAddress address) => GetAsset<AudioClip>(address.AudioClip);

    public Texture GetTexture(TextureAddress address) => GetAsset<Texture>(address.Texture);

    public TimedEffect GetEffectPrefab(EffectAddress address) =>
        GetAssetComponent<TimedEffect>(address.Effect);

    public Projectile GetProjectilePrefab(ProjectileAddress address) =>
        GetAssetComponent<Projectile>(address.Projectile);

    protected override void OnInitialize()
    {
      GetEffectPrefab(new EffectAddress
      {
        Effect = "Assets/ThirdParty/Hovl Studio/Magic circles/Prefabs/Magic circle 1 Variant.prefab"
      });
    }

    T GetAsset<T>(string address) where T : class
    {
      var op = Addressables.LoadAssetAsync<T>(address);
      var result = op.WaitForCompletion();
      return Errors.CheckNotNull(result);
    }

    T GetAssetComponent<T>(string address) where T : Component
    {
      return ComponentUtils.Get<T>(GetAsset<GameObject>(address));
    }
  }
}
