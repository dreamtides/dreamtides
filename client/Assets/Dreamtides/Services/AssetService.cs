#nullable enable

using Dreamtides.Components;
using Dreamtides.Schema;
using Dreamtides.Utils;
using UnityEngine;
using UnityEngine.AddressableAssets;

namespace Dreamtides.Services
{
  public class AssetService : Service
  {
    public Sprite GetSprite(SpriteAddress address) => GetAsset<Sprite>(address.Sprite);

    public Font GetFont(FontAddress address) => GetAsset<Font>(address.Font);

    public AudioClip GetAudioClip(AudioClipAddress address) => GetAsset<AudioClip>(address.AudioClip);

    public Material GetMaterial(MaterialAddress address) => GetAsset<Material>(address.Material);

    public TimedEffect GetEffectPrefab(EffectAddress address) =>
        GetAssetComponent<TimedEffect>(address.Effect);

    public Projectile GetProjectilePrefab(ProjectileAddress address) =>
        GetAssetComponent<Projectile>(address.Projectile);

    protected override void OnInitialize(TestMode testMode)
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
