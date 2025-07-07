#nullable enable

using System.Collections;
using System.Collections.Generic;
using Dreamtides.Layout;
using Dreamtides.Components;
using UnityEngine;

namespace Dreamtides.Services
{
  public class TestHelperService : Service
  {
    private Dictionary<MonoBehaviour, List<Vector3>> movementHistory = new();
    private HashSet<Displayable> lastKnownDisplayables = new();
    private HashSet<string> displayablesSeenDuringIdle = new();
    private bool isTrackingDisplayables = false;

    public bool DidObjectMove(MonoBehaviour obj)
    {
      if (movementHistory.TryGetValue(obj, out var positions))
      {
        return positions.Count > 1;
      }
      return false;
    }

    /// <summary>
    /// Returns the unity object names of all Displayable objects that were *not* present in
    /// the scene on the last call to WaitForIdle() but are now present in the scene.
    /// </summary>
    public List<string> CreatedDisplayables()
    {
      var result = new List<string>();

      if (isTrackingDisplayables)
      {
        var currentDisplayables = FindObjectsByType<Displayable>(FindObjectsSortMode.None);

        foreach (var displayable in currentDisplayables)
        {
          if (!lastKnownDisplayables.Contains(displayable))
          {
            result.Add(displayable.name);
          }
        }

        result.AddRange(displayablesSeenDuringIdle);
      }

      return result;
    }

    /// <summary>
    /// Start tracking displayables for later comparison in
    /// CreatedTemporaryDisplayables().
    /// </summary>
    public void StartTrackingDisplayables()
    {
      lastKnownDisplayables.Clear();
      displayablesSeenDuringIdle.Clear();
      isTrackingDisplayables = true;

      var currentDisplayables = FindObjectsByType<Displayable>(FindObjectsSortMode.None);
      foreach (var displayable in currentDisplayables)
      {
        lastKnownDisplayables.Add(displayable);
      }
    }

    public IEnumerator WaitForIdle(float timeoutSeconds)
    {
      var epsilon = 0.1f;
      var waitTime = 1.0f;
      var lastPositions = new Dictionary<MonoBehaviour, Vector3>();
      var timer = 0.0f;
      var totalTimer = 0.0f;
      movementHistory.Clear();

      void TrackObjects<T>() where T : MonoBehaviour
      {
        var objects = FindObjectsByType<T>(FindObjectsSortMode.None);
        foreach (var obj in objects)
        {
          lastPositions[obj] = obj.transform.position;
          if (!movementHistory.ContainsKey(obj))
          {
            movementHistory[obj] = new List<Vector3> { obj.transform.position };
          }
        }
      }

      bool CheckMovement<T>() where T : MonoBehaviour
      {
        var objects = FindObjectsByType<T>(FindObjectsSortMode.None);
        foreach (var obj in objects)
        {
          if (obj is TimedEffect or Projectile)
          {
            if (obj.gameObject.activeSelf)
            {
              return true;
            }
          }

          var currentPosition = obj.transform.position;

          if (lastPositions.TryGetValue(obj, out var lastPosition))
          {
            if (Vector3.Distance(currentPosition, lastPosition) > epsilon)
            {
              lastPositions[obj] = currentPosition;
              if (!movementHistory.ContainsKey(obj))
              {
                movementHistory[obj] = new List<Vector3> { currentPosition };
              }
              else
              {
                movementHistory[obj].Add(currentPosition);
              }
              return true;
            }
          }
          else
          {
            lastPositions[obj] = currentPosition;
            if (!movementHistory.ContainsKey(obj))
            {
              movementHistory[obj] = new List<Vector3> { currentPosition };
            }
            else
            {
              movementHistory[obj].Add(currentPosition);
            }
            return true;
          }
        }
        return false;
      }

      TrackObjects<Displayable>();
      TrackObjects<BattlefieldNumber>();
      TrackObjects<Projectile>();
      TrackObjects<TimedEffect>();

      yield return new WaitForEndOfFrame();

      while (timer < waitTime)
      {
        if (totalTimer >= timeoutSeconds)
        {
          throw new System.TimeoutException($"WaitForIdle exceeded timeout of {timeoutSeconds} seconds");
        }

        var hasMovement = false;

        hasMovement |= CheckMovement<Displayable>();
        hasMovement |= CheckMovement<BattlefieldNumber>();
        hasMovement |= CheckMovement<Projectile>();
        hasMovement |= CheckMovement<TimedEffect>();

        if (isTrackingDisplayables)
        {
          var currentDisplayables = FindObjectsByType<Displayable>(FindObjectsSortMode.None);
          foreach (var displayable in currentDisplayables)
          {
            if (!lastKnownDisplayables.Contains(displayable))
            {
              displayablesSeenDuringIdle.Add(displayable.name);
            }
          }
        }

        if (hasMovement)
        {
          timer = 0.0f;
        }
        else
        {
          timer += Time.deltaTime;
        }

        totalTimer += Time.deltaTime;
        yield return new WaitForEndOfFrame();
      }
    }
  }
}