#nullable enable

using UnityEngine;
using Newtonsoft.Json;
using System.Collections.Generic;
using TMPro;

namespace Dreamcaller.Services
{
    public class CommandService : MonoBehaviour
    {
        [SerializeField] TextMeshProUGUI _text;

        void Start()
        {
            _text.text = Plugin.GetScene(0).StatusDescription;
        }

        void Update()
        {

        }
    }
}