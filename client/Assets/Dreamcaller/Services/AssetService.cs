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

    public TimedEffect GetEffect(EffectAddress address) => GetAssetComponent<TimedEffect>(address.Effect);

    public Projectile GetProjectile(ProjectileAddress address) => GetAssetComponent<Projectile>(address.Projectile);

    T GetAsset<T>(string address) where T : class
    {
      var op = Addressables.LoadAssetAsync<T>("myGameObjectKey");
      var result = op.WaitForCompletion();
      return Errors.CheckNotNull(result);
    }

    T GetAssetComponent<T>(string address) where T : Component
    {
      return ComponentUtils.Get<T>(GetAsset<GameObject>(address));
    }
  }
}
