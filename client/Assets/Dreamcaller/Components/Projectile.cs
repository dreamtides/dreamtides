#nullable enable

using System.Collections;
using DG.Tweening;
using Dreamcaller.Schema;
using Dreamcaller.Services;
using Dreamcaller.Utils;
using UnityEngine;

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
      Milliseconds? duration,
      EffectAddress? additionalHit,
      Milliseconds? additionalHitDelay,
      AudioClipAddress? fireSound,
      AudioClipAddress? impactSound)
    {
      transform.localScale = _scale * Vector3.one;
      transform.LookAt(target);
      var rotation = Quaternion.LookRotation(transform.position - target.position);

      if (_flash)
      {
        var flash = registry.AssetPoolService.Create(_flash, transform.position);
        flash.transform.rotation = rotation;
        flash.transform.localScale = _scale * Vector3.one;
      }

      if (fireSound != null)
      {
        registry.MainAudioSource.PlayOneShot(registry.AssetService.GetAudioClip(fireSound));
      }
      else
      {
        registry.SoundService.PlayFireProjectileSound();
      }

      yield return TweenUtils.Sequence($"{name} Projectile")
        .Append(transform.DOMove(target.position, duration?.ToSeconds() ?? 0.3f).SetEase(Ease.Linear))
        .WaitForCompletion();

      TimedEffect? hit = null;
      if (_hit)
      {
        hit = registry.AssetPoolService.Create(_hit, transform.position);
        hit.transform.rotation = rotation;
        hit.transform.localScale = _scale * Vector3.one;
      }

      if (impactSound != null)
      {
        registry.MainAudioSource.PlayOneShot(registry.AssetService.GetAudioClip(impactSound));
      }
      else
      {
        registry.SoundService.PlayImpactProjectileSound();
      }

      gameObject.SetActive(value: false);

      if (additionalHit != null)
      {
        yield return new WaitForSeconds(additionalHitDelay?.ToSeconds() ?? 0);
        var additionalHitEffect =
          registry.AssetPoolService.Create(registry.AssetService.GetEffect(additionalHit), transform.position);
        additionalHitEffect.transform.rotation = rotation;

        if (hit)
        {
          hit!.gameObject.SetActive(false);
        }
      }
    }
  }
}