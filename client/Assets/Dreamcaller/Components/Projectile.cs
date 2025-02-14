#nullable enable

using System.Collections;
using DG.Tweening;
using Dreamcaller.Schema;
using Dreamcaller.Services;
using UnityEngine;
using UnityEngine.AddressableAssets;

namespace Dreamcaller.Components
{
  [DisallowMultipleComponent]
  public sealed class Projectile : MonoBehaviour
  {
    [SerializeField] float _scale = 3f;
    [SerializeField] TimedEffect? _flash;
    [SerializeField] TimedEffect? _hit;

    public void EditorSetEffects(TimedEffect? hit, TimedEffect? flash)
    {
      _hit = hit;
      _flash = flash;
    }

    public IEnumerator Fire(
      Registry registry,
      Transform target,
      TimeValue? duration,
      EffectAddress? additionalHit,
      TimeValue? additionalHitDelay,
      AudioClipAddress? fireSound,
      AudioClipAddress? impactSound)
    {
      transform.localScale = _scale * Vector3.one;
      transform.LookAt(target);
      var rotation = Quaternion.LookRotation(transform.position - target.position);
      yield return null;

      // if (_flash && _flash != null)
      // {
      //   var flash = registry.AssetPoolService.Create(_flash, transform.position);
      //   flash.transform.rotation = rotation;
      //   flash.transform.localScale = _scale * Vector3.one;
      // }

      // if (fireSound != null)
      // {
      //   AssetUtil.PlayOneShot(registry.MainAudioSource, registry.AssetService.GetAudioClip(fireSound));
      // }
      // else
      // {
      //   registry.StaticAssets.PlayFireProjectileSound();
      // }

      // yield return TweenUtils.Sequence($"{name} Projectile")
      //   .Append(transform.DOMove(target.position, DataUtils.ToSeconds(duration, 300)).SetEase(Ease.Linear))
      //   .WaitForCompletion();

      // TimedEffect? hit = null;
      // if (_hit && _hit != null)
      // {
      //   hit = registry.AssetPoolService.Create(_hit, transform.position);
      //   hit.transform.rotation = rotation;
      //   hit.transform.localScale = _scale * Vector3.one;
      // }

      // if (impactSound != null)
      // {
      //   AssetUtil.PlayOneShot(registry.MainAudioSource, registry.AssetService.GetAudioClip(impactSound));
      // }
      // else
      // {
      //   registry.StaticAssets.PlayImpactSound();
      // }

      // gameObject.SetActive(value: false);

      // if (additionalHit != null)
      // {
      //   yield return new WaitForSeconds(DataUtils.ToSeconds(additionalHitDelay, 0));
      //   var additionalHitEffect =
      //     registry.AssetPoolService.Create(registry.AssetService.GetEffect(additionalHit), transform.position);
      //   additionalHitEffect.transform.rotation = rotation;

      //   if (hit)
      //   {
      //     hit!.gameObject.SetActive(false);
      //   }
      // }
    }
  }
}