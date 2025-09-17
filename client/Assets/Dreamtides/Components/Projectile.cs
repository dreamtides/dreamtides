#nullable enable

using System;
using System.Collections;
using System.Collections.Generic;
using DG.Tweening;
using Dreamtides.Schema;
using Dreamtides.Services;
using Dreamtides.Utils;
using UnityEngine;

namespace Dreamtides.Components
{
  [DisallowMultipleComponent]
  public sealed class Projectile : MonoBehaviour
  {
    [SerializeField]
    float _scale = 3f;

    [SerializeField]
    public TimedEffect? _flash;

    [SerializeField]
    public TimedEffect? _hit;

    [SerializeField]
    public bool _useFirePointRotation;

    [SerializeField]
    public Vector3 _rotationOffset = new Vector3(0, 0, 0);

    [SerializeField]
    public ParticleSystem? _hitParticleSystem;

    [SerializeField]
    public Light? _light;

    [SerializeField]
    public List<GameObject> _detached = new();

    [SerializeField]
    public ParticleSystem? _projectileParticleSystem;

    public IEnumerator Fire(
      Registry registry,
      Transform target,
      Milliseconds? duration,
      EffectAddress? additionalHit = null,
      Milliseconds? additionalHitDelay = null,
      AudioClipAddress? fireSound = null,
      AudioClipAddress? impactSound = null,
      Action? onHit = null,
      bool mute = false
    )
    {
      if (_flash)
      {
        _flash.gameObject.SetActive(false);
        _flash.transform.parent = null;
      }
      if (_hit)
      {
        _hit.gameObject.SetActive(false);
        _hit.transform.parent = null;
      }

      transform.localScale = _scale * Vector3.one;
      transform.LookAt(target);
      var rotation = Quaternion.LookRotation(transform.position - target.position);

      if (_flash)
      {
        //var flash = registry.AssetPoolService.Create(_flash,
        //transform.position);
        _flash.transform.position = transform.position;
        _flash.gameObject.SetActive(true);
        _flash.transform.rotation = rotation;
        _flash.transform.localScale = _scale * Vector3.one;
      }

      if (fireSound != null)
      {
        registry.SoundService.Play(fireSound);
      }
      else if (!mute)
      {
        registry.SoundService.PlayFireProjectileSound();
      }

      yield return TweenUtils
        .Sequence($"{name} Projectile")
        .Append(
          transform.DOMove(target.position, duration?.ToSeconds() ?? 0.3f).SetEase(Ease.Linear)
        )
        .WaitForCompletion();

      if (_hit)
      {
        // hit = registry.AssetPoolService.Create(_hit, transform.position);
        _hit.transform.position = transform.position;
        _hit.gameObject.SetActive(true);
        _hit.transform.rotation = rotation;
        _hit.transform.localScale = _scale * Vector3.one;
        _hit.gameObject.SetActive(true);
      }

      onHit?.Invoke();

      if (impactSound != null)
      {
        registry.SoundService.Play(impactSound);
      }
      else if (!mute)
      {
        registry.SoundService.PlayImpactProjectileSound();
      }

      if (additionalHit != null)
      {
        yield return new WaitForSeconds(additionalHitDelay?.ToSeconds() ?? 0);
        if (_hit)
        {
          _hit.gameObject.SetActive(false);
        }
        var additionalHitEffect = registry.AssetPoolService.Create(
          registry.AssetService.GetEffectPrefab(additionalHit),
          transform.position
        );
        additionalHitEffect.transform.rotation = rotation;
      }

      gameObject.SetActive(value: false);
    }
  }
}
