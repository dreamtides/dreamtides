#nullable enable

using System.Collections.Generic;
using Dreamcaller.Schema;
using UnityEngine;

namespace Dreamcaller.Services
{
  public class SoundService : Service
  {
    [SerializeField] List<AudioClip> _cardSounds = null!;
    [SerializeField] List<AudioClip> _fireProjectileSounds = null!;
    [SerializeField] List<AudioClip> _impactProjectileSounds = null!;

    public void Play(AudioClip clip)
    {
      Registry.MainAudioSource.PlayOneShot(clip);
    }

    public void Play(AudioClipAddress clipAddress)
    {
      Registry.MainAudioSource.PlayOneShot(Registry.AssetService.GetAudioClip(clipAddress));
    }

    public void PlayCardSound()
    {
      PlayRandom(_cardSounds);
    }

    public void PlayFireProjectileSound()
    {
      PlayRandom(_fireProjectileSounds);
    }

    public void PlayImpactProjectileSound()
    {
      PlayRandom(_impactProjectileSounds);
    }

    void PlayRandom(List<AudioClip> clips)
    {
      if (clips.Count == 0)
      {
        return;
      }

      var randomIndex = Random.Range(0, clips.Count);
      var randomSound = clips[randomIndex];
      Play(randomSound);
    }
  }
}