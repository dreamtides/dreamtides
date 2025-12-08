#nullable enable

using System.Collections.Generic;
using System.Runtime.CompilerServices;
using Dreamtides.Schema;
using Dreamtides.Utils;
using UnityEngine;

[assembly: InternalsVisibleTo("Dreamtides.Tests")]

namespace Dreamtides.Services
{
  public abstract class SoundService : Service
  {
    public abstract void Play(AudioClip clip);
    public abstract void Play(AudioClipAddress clipAddress);
    public abstract void PlayCardSound();
    public abstract void PlayFireProjectileSound();
    public abstract void PlayImpactProjectileSound();
    public abstract void PlayWhooshSound();
    public abstract void PlayClickSound();
    public abstract void PlayDrawCardSound();
    public abstract void PlayMessageSound(GameMessageType messageType);
  }

  public class SoundServiceImpl : SoundService
  {
    [SerializeField]
    List<AudioClip> _cardSounds = null!;

    [SerializeField]
    List<AudioClip> _fireProjectileSounds = null!;

    [SerializeField]
    List<AudioClip> _impactProjectileSounds = null!;

    [SerializeField]
    List<AudioClip> _whooshSounds = null!;

    [SerializeField]
    List<AudioClip> _clickSounds = null!;

    [SerializeField]
    List<AudioClip> _drawCardSounds = null!;

    [SerializeField]
    internal AudioClip _yourTurnSound = null!;

    [SerializeField]
    internal AudioClip _enemyTurnSound = null!;

    [SerializeField]
    internal AudioClip _victorySound = null!;

    [SerializeField]
    internal AudioClip _defeatSound = null!;

    protected override void OnInitialize(GameMode _mode, TestConfiguration? testConfiguration)
    {
      if (testConfiguration != null)
      {
        Registry.MainAudioSource.volume = 0.0f;
      }
    }

    public override void Play(AudioClip clip)
    {
      Registry.MainAudioSource.PlayOneShot(clip);
    }

    public override void Play(AudioClipAddress clipAddress)
    {
      Registry.MainAudioSource.PlayOneShot(Registry.AssetService.GetAudioClip(clipAddress));
    }

    public override void PlayCardSound()
    {
      PlayRandom(_cardSounds);
    }

    public override void PlayFireProjectileSound()
    {
      PlayRandom(_fireProjectileSounds);
    }

    public override void PlayImpactProjectileSound()
    {
      PlayRandom(_impactProjectileSounds);
    }

    public override void PlayWhooshSound()
    {
      PlayRandom(_whooshSounds);
    }

    public override void PlayClickSound()
    {
      PlayRandom(_clickSounds);
    }

    public override void PlayDrawCardSound()
    {
      PlayRandom(_drawCardSounds);
    }

    public override void PlayMessageSound(GameMessageType messageType)
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
