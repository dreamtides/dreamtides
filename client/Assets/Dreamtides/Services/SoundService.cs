#nullable enable

using System.Collections.Generic;
using Dreamtides.Schema;
using Dreamtides.Utils;
using UnityEngine;

namespace Dreamtides.Services
{
  public class SoundService : Service
  {
    [SerializeField] List<AudioClip> _cardSounds = null!;
    [SerializeField] List<AudioClip> _fireProjectileSounds = null!;
    [SerializeField] List<AudioClip> _impactProjectileSounds = null!;
    [SerializeField] List<AudioClip> _whooshSounds = null!;
    [SerializeField] List<AudioClip> _clickSounds = null!;
    [SerializeField] AudioClip _yourTurnSound = null!;
    [SerializeField] AudioClip _enemyTurnSound = null!;
    [SerializeField] AudioClip _victorySound = null!;
    [SerializeField] AudioClip _defeatSound = null!;

    protected override void OnInitialize(TestConfiguration? testConfiguration)
    {
      if (testConfiguration != null)
      {
        Registry.Layout.MainAudioSource.volume = 0.0f;
      }
    }

    public void Play(AudioClip clip)
    {
      Registry.Layout.MainAudioSource.PlayOneShot(clip);
    }

    public void Play(AudioClipAddress clipAddress)
    {
      Registry.Layout.MainAudioSource.PlayOneShot(Registry.AssetService.GetAudioClip(clipAddress));
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

    public void PlayWhooshSound()
    {
      PlayRandom(_whooshSounds);
    }

    public void PlayClickSound()
    {
      PlayRandom(_clickSounds);
    }

    public void PlayMessageSound(GameMessageType messageType)
    {
      switch (messageType)
      {
        case GameMessageType.YourTurn:
          Play(_yourTurnSound);
          break;
        case GameMessageType.EnemyTurn:
          Play(_enemyTurnSound);
          break;
        case GameMessageType.Victory:
          Play(_victorySound);
          break;
        case GameMessageType.Defeat:
          Play(_defeatSound);
          break;
        default:
          throw Errors.UnknownEnumValue(messageType);
      }
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