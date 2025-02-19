#nullable enable

using System.Collections.Generic;
using UnityEngine;
using System.Collections;
using System.Linq;

namespace Dreamcaller.Services
{
  public class MusicService : Service
  {
    [SerializeField] List<AudioClip> _tracks = null!;
    [SerializeField] AudioSource _musicAudioSource = null!;
    [SerializeField] float _crossFadeDuration = 2f;

    List<int> _shuffledIndices = new();
    int _currentTrackIndex = -1;
    Coroutine? _trackMonitorCoroutine;

    protected override void OnInitialize()
    {
      ShufflePlaylist();
      PlayNextTrack();
    }

    void ShufflePlaylist()
    {
      _shuffledIndices = Enumerable.Range(0, _tracks.Count).ToList();
      for (int i = _shuffledIndices.Count - 1; i > 0; i--)
      {
        int randomIndex = Random.Range(0, i + 1);
        (_shuffledIndices[i], _shuffledIndices[randomIndex]) = (_shuffledIndices[randomIndex], _shuffledIndices[i]);
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

      if (_trackMonitorCoroutine != null)
      {
        StopCoroutine(_trackMonitorCoroutine);
      }
      _trackMonitorCoroutine = StartCoroutine(MonitorTrackCompletion());
    }

    IEnumerator CrossfadeToTrack(AudioClip nextTrack)
    {
      float startVolume = _musicAudioSource.volume;
      float elapsed = 0;

      // Fade out current track if playing
      while (elapsed < _crossFadeDuration && _musicAudioSource.isPlaying)
      {
        elapsed += Time.deltaTime;
        _musicAudioSource.volume = Mathf.Lerp(startVolume, 0, elapsed / _crossFadeDuration);
        yield return null;
      }

      // Switch to new track
      _musicAudioSource.clip = nextTrack;
      _musicAudioSource.Play();
      elapsed = 0;

      // Fade in new track
      while (elapsed < _crossFadeDuration)
      {
        elapsed += Time.deltaTime;
        _musicAudioSource.volume = Mathf.Lerp(0, startVolume, elapsed / _crossFadeDuration);
        yield return null;
      }
    }

    IEnumerator MonitorTrackCompletion()
    {
      yield return new WaitUntil(() =>
        !_musicAudioSource.isPlaying ||
        (_musicAudioSource.clip && _musicAudioSource.time >= _musicAudioSource.clip.length - _crossFadeDuration));

      PlayNextTrack();
    }
  }
}
