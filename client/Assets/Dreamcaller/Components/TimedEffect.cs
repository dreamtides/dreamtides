#nullable enable

using System;
using System.Collections;
using UnityEngine;

namespace Dreamcaller.Components
{
  [DisallowMultipleComponent]
  public sealed class TimedEffect : MonoBehaviour
  {
    [SerializeField] float _duration;
    [SerializeField] bool _looping;
    public Action? OnDisable { get; set; }

    void OnEnable()
    {
      if (!_looping)
      {
        StartCoroutine(DisableAsync(_duration));
      }
    }

    public void SetStartColor(Color color)
    {
      foreach (var ps in GetComponentsInChildren<ParticleSystem>())
      {
        var main = ps.main;
        var current = main.startColor.color;
        Color.RGBToHSV(current, out _, out var s, out var v);
        if (s > 0.2f && v > 0.2f)
        {
          main.startColor = new Color(color.r, color.g, color.b, current.a);
        }
      }
    }

    void OnValidate()
    {
      _duration = 0.0f;

      foreach (var system in GetComponentsInChildren<ParticleSystem>())
      {
        var main = system.main;
        _duration = Mathf.Max(_duration, main.duration + main.startLifetime.constantMax);
      }

      foreach (var audioSource in GetComponentsInChildren<AudioSource>())
      {
        if (audioSource.clip != null)
        {
          _duration = Mathf.Max(_duration, audioSource.clip.length);
        }
      }
    }

    IEnumerator DisableAsync(float duration)
    {
      // Add a little extra time for safety
      yield return new WaitForSeconds(duration + 0.5f);
      gameObject.SetActive(value: false);
      OnDisable?.Invoke();
    }
  }
}