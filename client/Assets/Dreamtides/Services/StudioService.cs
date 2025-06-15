#nullable enable
using System;
using System.Collections;
using System.Collections.Generic;
using System.Linq;
using Dreamtides.Components;
using Dreamtides.Utils;
using UnityEngine;

namespace Dreamtides.Services
{
  public class StudioService : Service
  {
    [SerializeField] Studio _studioPrefab = null!;
    [SerializeField] Transform _studioPosition = null!;

    private Dictionary<int, CaptureSession> _activeSessions = new();

    private class CaptureSession
    {
      public Studio Studio = null!;
      public GameObject Subject = null!;
      public RenderTexture RenderTexture = null!;
      public int OutputInstanceId;
    }

    /// <summary>
    /// Captures a live image of a subject prefab and displays it on a
    /// RenderTexture on the provided MeshRenderer.
    /// </summary>
    public void CaptureSubject(GameObject prefab, MeshRenderer output)
    {
      var studio = ComponentUtils.Instantiate(_studioPrefab);
      var studioPosition = FindStudioPosition();
      studio.transform.SetParent(_studioPosition);
      studio.transform.position = studioPosition;

      var renderTexture = new RenderTexture(1024, 1024, 24);
      studio.StudioCamera.targetTexture = renderTexture;

      var instance = Instantiate(prefab);
      instance.transform.SetParent(studio.SubjectPosition);
      instance.transform.localPosition = Vector3.zero;
      instance.transform.localRotation = Quaternion.identity;

      output.material.mainTexture = renderTexture;

      var outputInstanceId = output.GetInstanceID();
      var session = new CaptureSession
      {
        Studio = studio,
        Subject = instance,
        RenderTexture = renderTexture,
        OutputInstanceId = outputInstanceId
      };

      _activeSessions[outputInstanceId] = session;
    }

    /// <summary>
    /// Ends a capture session for a MeshRenderer based on its instance ID.
    /// </summary>
    public void EndCapture(MeshRenderer output)
    {
      var outputInstanceId = output.GetInstanceID();

      if (_activeSessions.TryGetValue(outputInstanceId, out var session))
      {
        if (session.Studio != null)
        {
          Destroy(session.Studio.gameObject);
        }

        if (session.Subject != null)
        {
          Destroy(session.Subject);
        }

        if (session.RenderTexture != null)
        {
          session.RenderTexture.Release();
          Destroy(session.RenderTexture);
        }

        _activeSessions.Remove(outputInstanceId);
      }
    }

    private Vector3 FindStudioPosition()
    {
      var basePosition = _studioPosition.position;

      if (_activeSessions.Count == 0)
      {
        return basePosition;
      }

      var occupiedXPositions = _activeSessions.Values
        .Where(s => s.Studio != null)
        .Select(s => s.Studio.transform.position.x)
        .ToList();

      var minX = occupiedXPositions.Min();
      var newX = minX - 50f;

      return new Vector3(newX, basePosition.y, basePosition.z);
    }
  }
}