#nullable enable

using Dreamcaller.Components;
using UnityEditor;
using UnityEngine;

namespace Dreamcaller.Editors
{
    [CustomEditor(typeof(TimedEffect))]
    public sealed class TimedEffectEditor : Editor
    {
        Color _color = Color.clear;

        public override void OnInspectorGUI()
        {
            DrawDefaultInspector();

            if (GUILayout.Button("Disabling Looping"))
            {
                foreach (var particleSystem in ((TimedEffect)target).GetComponentsInChildren<ParticleSystem>())
                {
                    var main = particleSystem.main;
                    main.loop = false;
                }
            }

            if (GUILayout.Button("Hierarchy Scaling"))
            {
                foreach (var particleSystem in ((TimedEffect)target).GetComponentsInChildren<ParticleSystem>())
                {
                    var main = particleSystem.main;
                    main.scalingMode = ParticleSystemScalingMode.Hierarchy;
                }
            }

            if (GUILayout.Button("Duration 1s"))
            {
                foreach (var particleSystem in ((TimedEffect)target).GetComponentsInChildren<ParticleSystem>())
                {
                    particleSystem.Stop();
                    var main = particleSystem.main;
                    main.duration = Mathf.Min(main.duration, 1f);
                    var startLifetime = main.startLifetime;
                    startLifetime.constantMax = Mathf.Min(startLifetime.constantMax, 1f);
                    main.startLifetime = startLifetime;
                }
            }

            var color = EditorGUILayout.ColorField("Main Color", _color);
            if (color != _color && color != Color.clear)
            {
                _color = color;
            }

            if (GUILayout.Button("Set Start Color") && _color != Color.clear)
            {
                foreach (var particleSystem in ((TimedEffect)target).GetComponentsInChildren<ParticleSystem>())
                {
                    var main = particleSystem.main;
                    main.startColor = _color;
                }

                _color = Color.clear;
            }
        }
    }
}