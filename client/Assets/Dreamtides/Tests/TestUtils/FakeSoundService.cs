#nullable enable

using System.Collections.Generic;
using Dreamtides.Schema;
using Dreamtides.Services;
using UnityEngine;

namespace Dreamtides.Tests.TestUtils
{
  public class FakeSoundService : SoundService
  {
    readonly List<string> _playedSounds = new();

    public IReadOnlyList<string> PlayedSounds => _playedSounds;

    public void ClearPlayedSounds()
    {
      _playedSounds.Clear();
    }

    public override void Play(AudioClip clip)
    {
      _playedSounds.Add(clip?.name ?? "null");
    }

    public override void Play(AudioClipAddress clipAddress)
    {
      _playedSounds.Add($"Address:{clipAddress}");
    }

    public override void PlayCardSound()
    {
      _playedSounds.Add("CardSound");
    }

    public override void PlayFireProjectileSound()
    {
      _playedSounds.Add("FireProjectileSound");
    }

    public override void PlayImpactProjectileSound()
    {
      _playedSounds.Add("ImpactProjectileSound");
    }

    public override void PlayWhooshSound()
    {
      _playedSounds.Add("WhooshSound");
    }

    public override void PlayClickSound()
    {
      _playedSounds.Add("ClickSound");
    }

    public override void PlayDrawCardSound()
    {
      _playedSounds.Add("DrawCardSound");
    }

    public override void PlayMessageSound(GameMessageType messageType)
    {
      _playedSounds.Add($"MessageSound:{messageType}");
    }
  }
}
