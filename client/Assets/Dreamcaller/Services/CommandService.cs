using UnityEngine;
using Newtonsoft.Json;
using System.Collections.Generic;
using TMPro;

class Account
{
    public string Email { get; set; }
    public bool Active { get; set; }
    public IList<string> Roles { get; set; }
}

public class CommandService : MonoBehaviour
{
    [SerializeField] TextMeshProUGUI _text;

    // Start is called once before the first execution of Update after the MonoBehaviour is created
    void Start()
    {
        string json = @"{
            'Email': 'james@example.com',
            'Active': true,
            'CreatedDate': '2013-01-20T00:00:00Z',
            'Roles': [
                'User',
                'Admin'
            ]
        }";
        Account account = JsonConvert.DeserializeObject<Account>(json);
        //_text.text = Plugin.ReturnTwo().ToString() + " " + account.Email;
        _text.text = Plugin.GetScene(0).StatusDescription;
    }

    // Update is called once per frame
    void Update()
    {

    }
}
