#nullable enable
using System;
using System.Collections;
using System.Collections.Generic;
using System.Linq;
using System.Runtime.CompilerServices;
using Dreamtides.Components;
using Dreamtides.Schema;
using Dreamtides.Utils;
using UnityEngine;

[assembly: InternalsVisibleTo("Dreamtides.Tests")]

namespace Dreamtides.Services
{
  public class StudioService : Service
  {
    [SerializeField]
    internal Studio _studioPrefab = null!;

    [SerializeField]
    internal Transform _studioPosition = null!;

    [SerializeField]
    List<CaptureSession> _captureSessions = new();

    private Dictionary<StudioType, CaptureSession> _activeSessions = new();

    [Serializable]
    private class CaptureSession
    {
      public Studio Studio = null!;
      public GameObject Subject = null!;
      public RenderTexture RenderTexture = null!;
      public StudioType StudioType;
      public string? CurrentPrimaryAnimation;
      public StudioAnimation? CurrentExitAnimation;
      public Coroutine? AnimationSequence;
    }

    /// <summary>
    /// Captures a live image of a subject prefab and displays it on a
    /// RenderTexture on the provided MeshRenderer.
    /// </summary>
    public void CaptureSubject(
      StudioType type,
      GameObject prefab,
      Renderer output,
      bool far = false
    )
    {
      Debug.Log("CaptureSubject: " + type);

      if (_activeSessions.TryGetValue(type, out var existingSession))
      {
        if (existingSession.AnimationSequence != null)
        {
          StopCoroutine(existingSession.AnimationSequence);
        }

        if (existingSession.Subject != null)
        {
          Destroy(existingSession.Subject);
        }

        var instance = Instantiate(prefab);
        instance.transform.SetParent(
          far ? existingSession.Studio.FarSubjectPosition : existingSession.Studio.SubjectPosition
        );
        instance.transform.localPosition = Vector3.zero;
        instance.transform.localRotation = Quaternion.identity;

        output.material.mainTexture = existingSession.RenderTexture;

        existingSession.Subject = instance;
        existingSession.CurrentPrimaryAnimation = null;
        existingSession.CurrentExitAnimation = null;
      }
      else
      {
        var studio = ComponentUtils.Instantiate(_studioPrefab);
        studio.StudioType = type;
        studio.IsFar = far;
        var studioPosition = FindStudioPosition();
        studio.transform.SetParent(_studioPosition);
        studio.transform.position = studioPosition;

        var renderTexture = new RenderTexture(1024, 1024, 24);
        studio.StudioCamera.targetTexture = renderTexture;

        var instance = Instantiate(prefab);
        instance.transform.SetParent(far ? studio.FarSubjectPosition : studio.SubjectPosition);
        instance.transform.localPosition = Vector3.zero;
        instance.transform.localRotation = Quaternion.identity;

        output.material.mainTexture = renderTexture;

        var session = new CaptureSession
        {
          Studio = studio,
          Subject = instance,
          RenderTexture = renderTexture,
          StudioType = type,
        };

        _activeSessions[type] = session;
        _captureSessions.Add(session);
      }
    }

    public void PlayStudioAnimation(PlayStudioAnimationCommand command)
    {
      if (_activeSessions.TryGetValue(command.StudioType, out var session))
      {
        if (session.AnimationSequence != null)
        {
          StopCoroutine(session.AnimationSequence);
        }

        session.AnimationSequence = StartCoroutine(PlayAnimationSequence(session, command));
      }
    }

    public GameObject? GetSubject(StudioType type)
    {
      if (_activeSessions.TryGetValue(type, out var session))
      {
        return session.Subject;
      }
      return null;
    }

    private IEnumerator PlayAnimationSequence(
      CaptureSession session,
      PlayStudioAnimationCommand command
    )
    {
      var animator = session.Subject.GetComponent<Animator>();

      if (session.CurrentPrimaryAnimation != null && session.CurrentExitAnimation != null)
      {
        animator.Play(session.CurrentExitAnimation.Name);
        yield return new WaitForEndOfFrame();
        var exitAnimationLength = animator.GetCurrentAnimatorStateInfo(0).length;
        yield return new WaitForSeconds(exitAnimationLength);
      }

      if (command.EnterAnimation != null)
      {
        animator.Play(command.EnterAnimation.Name);
        yield return new WaitForEndOfFrame();
        var enterAnimationLength = animator.GetCurrentAnimatorStateInfo(0).length;
        yield return new WaitForSeconds(enterAnimationLength);
      }

      animator.Play(command.Animation.Name);
      session.CurrentPrimaryAnimation = command.Animation.Name;
      session.CurrentExitAnimation = command.ExitAnimation;
      session.AnimationSequence = null;
    }

    private Vector3 FindStudioPosition()
    {
      var basePosition = _studioPosition.position;

      if (_activeSessions.Count == 0)
      {
        return basePosition;
      }

      var occupiedXPositions = _activeSessions
        .Values.Where(s => s.Studio != null)
        .Select(s => s.Studio.transform.position.x)
        .ToList();

      var minX = occupiedXPositions.Min();
      var newX = minX - 50f;

      return new Vector3(newX, basePosition.y, basePosition.z);
    }
  }
}
