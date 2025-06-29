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

    public bool DidObjectMove(MonoBehaviour obj)
    {
      if (movementHistory.TryGetValue(obj, out var positions))
      {
        return positions.Count > 1;
      }
      return false;
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