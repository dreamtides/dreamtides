#nullable enable

using System.Collections;
using System.Collections.Generic;
using System.Linq;
using UnityEngine;

namespace Dreamtides.Services
{
  public class MusicService : Service
  {
    [SerializeField]
    List<AudioClip> _tracks = null!;

    [SerializeField]
    float _crossFadeDuration = 2f;

    [SerializeField]
    List<int> _shuffledIndices = new();

    [SerializeField]
    int _currentTrackIndex = -1;
    bool _isTransitioning = false;

    protected override void OnInitialize(TestConfiguration? testConfiguration)
    {
      if (testConfiguration == null)
      {
        // Don't play music in tests
        ShufflePlaylist();
        PlayNextTrack();
      }
    }

    public void Mute()
    {
      Registry.Layout.MusicAudioSource.volume = 0;
    }

    public void Unmute()
    {
      Registry.Layout.MusicAudioSource.volume = 1;
    }

    void ShufflePlaylist()
    {
      _shuffledIndices = Enumerable.Range(0, _tracks.Count).ToList();
      for (int i = _shuffledIndices.Count - 1; i > 0; i--)
      {
        int randomIndex = Random.Range(0, i + 1);
        (_shuffledIndices[i], _shuffledIndices[randomIndex]) = (
          _shuffledIndices[randomIndex],
          _shuffledIndices[i]
        );
      }
      _currentTrackIndex = -1;
    }

    void PlayNextTrack()
    {
      _currentTrackIndex++;
      if (_currentTrackIndex >= _shuffledIndices.Count)
      {
        ShufflePlaylist();
        _currentTrackIndex = 0;
      }

      var nextTrack = _tracks[_shuffledIndices[_currentTrackIndex]];
      StartCoroutine(CrossfadeToTrack(nextTrack));
    }

    protected override void OnUpdate()
    {
      if (!_isTransitioning && Registry.Layout.MusicAudioSource.clip != null)
      {
        var shouldTransition =
          !Registry.Layout.MusicAudioSource.isPlaying
          || Registry.Layout.MusicAudioSource.time
            >= Registry.Layout.MusicAudioSource.clip.length - _crossFadeDuration;

        if (shouldTransition)
        {
          _isTransitioning = true;
          PlayNextTrack();
        }
      }
    }

    IEnumerator CrossfadeToTrack(AudioClip nextTrack)
    {
      float startVolume = Registry.Layout.MusicAudioSource.volume;
      float elapsed = 0;

      while (elapsed < _crossFadeDuration && Registry.Layout.MusicAudioSource.isPlaying)
      {
        elapsed += Time.deltaTime;
        Registry.Layout.MusicAudioSource.volume = Mathf.Lerp(
          startVolume,
          0,
          elapsed / _crossFadeDuration
        );
        yield return null;
      }

      Registry.Layout.MusicAudioSource.clip = nextTrack;
      Registry.Layout.MusicAudioSource.Play();
      elapsed = 0;

      while (elapsed < _crossFadeDuration)
      {
        elapsed += Time.deltaTime;
        Registry.Layout.MusicAudioSource.volume = Mathf.Lerp(
          0,
          startVolume,
          elapsed / _crossFadeDuration
        );
        yield return null;
      }

      _isTransitioning = false;
    }
  }
}
