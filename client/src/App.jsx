import { createEffect, createSignal, onMount } from 'solid-js';
import styles from './App.module.css';
import axios from "axios"
import * as Msal from "msal";

import { v4 as uuidv4 } from 'uuid';

const [response, setResponse] = createSignal([]);

const App = () => {
  const [email, setEmail] = createSignal("test1@yopmail.com");
  const [otp, setOTP] = createSignal("");
  const [methodId, setSetMethodId] = createSignal("");
  const [accesstoken, setAccessToken] = createSignal("");

  const [userId, setUserId] = createSignal("");
  const [firstName, setFirstName] = createSignal("");
  const [lastName, setLastName] = createSignal("");
  
  const [roomName, setRoomName] = createSignal("Test Workspace");
  const [role, setRole] = createSignal("owner");
  const [desc, setDesc] = createSignal("This is test workspace");

  const [wsId, setWsId] = createSignal("");
  const [peerId, setPeerId] = createSignal("");
  const [peerRole, setPeerRole] = createSignal("admin");

  const [inviteeFirstName, setInviteeFirstName] = createSignal("Vish");
  const [inviteeLastName, setInviteeLastName] = createSignal("Test");
  const [inviteeEmail, setInviteeEmail] = createSignal("vish@bhuman.ai");
  const [inviteePhone, setInviteePhone] = createSignal("");

  const [error, setError] = createSignal("");
  const [login, setLogin] = createSignal(false);
  const [createdWs, setCreatedWs] = createSignal(false);
  const [canInvite, setCanInvite] = createSignal(false);

  // Google
  let tokenClient;

  const CLIENT_ID = '368853664549-99bdqmr4u31p0n0vubgctsmcrstdrl58.apps.googleusercontent.com';
  const API_KEY = 'AIzaSyD4DfPABzWoK58vYcrhPaoVSrKSlqreLzs';
  const DISCOVERY_DOC = 'https://www.googleapis.com/discovery/v1/apis/people/v1/rest';
  const SCOPES = 'https://www.googleapis.com/auth/contacts.readonly';

  // Outlook
  let myMSALObj;

  const msalConfig = {
    auth: {
      clientId: '72c7d826-d6f3-4aaf-a3f5-6ba648d91661',
      // clientSecret: '7cOYbDp7KWCgeoJWDjjUqF3',
      // authority: 'https://login.microsoftonline.com/common',
      redirectUri: "http://localhost:3000/authorize",
    },
    cache: {
      cacheLocation: "sessionStorage", // This configures where your cache will be stored
      storeAuthStateInCookie: false, // Set this to "true" if you are having issues on IE11 or Edge
    }
  };

  const loginRequest = {
    scopes: ['openid', 'profile', 'User.Read', 'Contacts.Read']
  };

  // When the app mounts, do the following:
  onMount(() => {
    myMSALObj = new Msal.UserAgentApplication(msalConfig);
    gapi.load('client', startGoogleClient);
  });

  function webSockectConnect() {

    let ws = new WebSocket('ws://127.0.0.1:5000/ws');
    ws.onmessage = (e) => {
      const resJson = JSON.parse(e.data);
      setResponse([...response(), resJson]);
    };

    function connect() {

      ws.onopen = () => {
        console.log('Connected to server');
        clearTimeout(check);

        ws.send(JSON.stringify({ user_id: userId(), message_type: "connect", message: "" }));
      };

      ws.onclose = () => {
        console.log('Socket is closed. Reconnect will be attempted in 5 seconds.');
        setTimeout(function () {
          check();
        }, 5000);
      };

      ws.onerror = (error) => {
        console.error("Socket encounted error: ", error, "Closing socket");
        setInterval(check, 5000);
        ws.close();
      }
    }

    var check = () => {
      console.log("Checking if socket is closed");
      if (ws.readyState === WebSocket.CLOSED) {
        console.log("Socket is closed. Reconnect will be attempted in 5 second.");
        setTimeout(function () {
          connect();
        }, 5000)

        // In case of error, clear the interval
      } else if (ws.readyState === WebSocket.CLOSING) {
        console.log("Socket is closing");
        clearInterval(check);
      }
    }

    connect();
  }

  async function startGoogleClient() {
    await gapi.client.init({
      apiKey: API_KEY,
      discoveryDocs: [DISCOVERY_DOC],
    });

    tokenClient = google.accounts.oauth2.initTokenClient({
      client_id: CLIENT_ID,
      scope: SCOPES,
      callback: '', // defined later
    });
  }

  const sendSignup = () => {
    var param = {
      email: email(),
    }

    axios.post('http://127.0.0.1:4004/api/auth/email', param).then(res => {
      console.log(res);
      setResponse([...response(), res.data]);

      setSetMethodId(res.data.result.method_id);
    }
    ).catch(err => {
      console.log("Error:" + err);
      setResponse([...response(), err]);
    });
  }

  const verifyUser = () => {
    var param = {
      code: otp(),
      method_id: methodId()
    }

    axios.post('http://127.0.0.1:4004/api/verify/email', param).then(res => {
      console.log(res);
      setResponse([...response(), res.data]);

      setUserId(res.data.result.user_id);
      setAccessToken(res.data.result.token.access_token);

      const config = {
        headers: { Authorization: `Bearer ${accesstoken()}` }
      };
  
      let url = 'http://127.0.0.1:4000/api/user';
      axios.get(url, config).then(res => {
        console.log(res);
        setResponse([...response(), res.data]);

        if (res.data.code == 200) {
          setLogin(true);
          webSockectConnect();
        }        
      }
      ).catch(err => {
        console.log("Error:" + err);
        setResponse([...response(), err]);
      });      
    }
    ).catch(err => {
      console.log("Error:" + err);
      setResponse([...response(), err]);
    });
  }

  const createUser = () => {
    var param = {
      first_name: firstName(),
      last_name: lastName(),
      email: email(),
    }

    const config = {
      headers: { Authorization: `Bearer ${accesstoken()}` }
    };

    axios.post('http://127.0.0.1:4000/api/user', param, config).then(res => {
      console.log(res);
      setResponse([...response(), res.data]);

      setLogin(true);
      webSockectConnect();

      var param = {
        hash: "",
        account: email()
      }

      const config = {
        headers: { Authorization: `Bearer ${accesstoken()}` }
      };

      axios.put('http://127.0.0.1:4002/api/invite', param, config).then(res => {
        setResponse([...response(), res.data]);
      }).catch(err => {
        setResponse([...response(), err]);
      });
    }
    ).catch(err => {
      console.log("Error:" + err);
      setResponse([...response(), err]);
    });
  }

  const createWorkspace = () => {
    var param = {
      name: firstName(),
      role: lastName(),
      description: "test",
    }

    const config = {
      headers: { Authorization: `Bearer ${accesstoken()}` }
    };

    axios.post('http://127.0.0.1:4001/api/workspace', param, config).then(res => {
      console.log(res);
      setResponse([...response(), res.data]);
      setCreatedWs(true);
    }
    ).catch(err => {
      console.log("Error:" + err);
      setResponse([...response(), err]);
    });
  }

  const getProfile = () => {
    const config = {
      headers: { Authorization: `Bearer ${accesstoken()}` }
    };

    let url = 'http://127.0.0.1:4000/api/user';
    axios.get(url, config).then(res => {
      console.log(res);
      setResponse([...response(), res.data]);
    }
    ).catch(err => {
      console.log("Error:" + err);
      setResponse([...response(), err]);
    });
  }

  const deleteUser = () => {
    const config = {
      headers: { Authorization: `Bearer ${accesstoken()}` }
    };

    let url = 'http://127.0.0.1:4000/api/user';
    axios.delete(url, config).then(res => {
      console.log(res);
      setResponse([...response(), res.data]);
      setCanInvite(false);
      setCreatedWs(false);
      setLogin(false);
    }
    ).catch(err => {
      console.log("Error:" + err);
      setResponse([...response(), err]);
    });
  }

  const addToWorkspace = () => {
    var param = {
      id: wsId(),
      peer_id: peerId(),      
      role: peerRole(),
    }

    const config = {
      headers: { Authorization: `Bearer ${accesstoken()}` }
    };

    axios.post('http://127.0.0.1:4001/api/workspace_util', param, config).then(res => {
      console.log(res);
      setResponse([...response(), res.data]);
    }
    ).catch(err => {
      console.log("Error:" + err);
      setResponse([...response(), err]);
    });
  }

  async function listConnectionNames() {
    let response;
    try {
      // Fetch first 10 files
      response = await gapi.client.people.people.connections.list({
        'resourceName': 'people/me',
        'pageSize': 100,
        'personFields': 'addresses,birthdays,emailAddresses,genders,names,organizations,phoneNumbers,photos,userDefined',
      });
    } catch (err) {
      return err.message;
    }
    const connections = response.result.connections;
    if (!connections || connections.length == 0) {
      return 'No connections found.';
    }
    // Flatten to string to display
    const output = connections.reduce(
      (str, person) => {
        if (!person.names || person.names.length === 0) {
          return `${str}Missing display name\n`;
        }
        return `${str}${person.names[0].displayName}\n`;
      },
      'Connections:\n');
    return output;
  }

  const syncGoogleContacts = async () => {
    tokenClient.callback = async (resp) => {
      if (resp.error !== undefined) {
        throw (resp);
      }

      console.log("token = ", resp.access_token);

      var param = {
        provider: 'google',
        email: email(),
        phone: '',
        token: resp.access_token,
      }

      const config = {
        headers: { Authorization: `Bearer ${accesstoken()}` }
      };

      axios.post('http://127.0.0.1:4003/api/contacts', param, config).then(res => {
        setResponse([...response(), res.data]);
        setCanInvite(true);
      }).catch(err => {
        setResponse([...response(), err]);
      });

      //let output = await listConnectionNames();
      //setResponse([...response(), output]);
    };

    if (gapi.client.getToken() === null) {
      // Prompt the user to select a Google Account and ask for consent to share their data
      // when establishing a new session.
      tokenClient.requestAccessToken({ prompt: 'consent' });
    } else {
      // Skip display of account chooser and consent dialog for an existing session.
      tokenClient.requestAccessToken({ prompt: '' });
    }
  }

  const syncOutlookContacts = async () => {
    myMSALObj.loginPopup(loginRequest)
      .then(res => {
        if (myMSALObj.getAccount()) {
          var tokenRequest = {
            scopes: ['User.Read', 'Contacts.Read']
          };
          myMSALObj.acquireTokenSilent(tokenRequest)
            .then(res => {
              var param = {
                provider: 'outlook',
                email: email(),
                phone: '1234567890',
                token: res.accessToken,
              }

              const config = {
                headers: { Authorization: `Bearer ${accesstoken()}` }
              };

              axios.post('http://127.0.0.1:4003/api/contacts', param, config).then(res => {
                setResponse([...response(), res.data]);
                setCanInvite(true);
              }).catch(err => {
                setResponse([...response(), err]);
              });
            })
            .catch(err => {
              setResponse([...response(), err]);

              if (err.name === "InteractionRequiredAuthError") {
                return myMSALObj.acquireTokenPopup(tokenRequest)
                  .then(res => {
                    console.log(res);

                    var param = {
                      provider: 'outlook',
                      email: email(),
                      phone: '',
                      token: res.accessToken,
                    }

                    const config = {
                      headers: { Authorization: `Bearer ${accesstoken()}` }
                    };

                    axios.post('http://127.0.0.1:4003/api/contacts', param, config).then(res => {
                      setResponse([...response(), res.data]);
                      setCanInvite(true);
                    }).catch(err => {
                      setResponse([...response(), err]);
                    });
                  })
                  .catch(err => {
                    // handle error
                  });
              }
            });
        }
      }).catch(error => {
        setResponse([...response(), error]);
      });
  }

  const sendInvite = async () => {
    var param = {
      sender: {
        first_name: firstName(),
        last_name: lastName()
      },
      receivers: [
        {
          first_name: inviteeFirstName(),
          last_name: inviteeLastName(),
          email: inviteeEmail(),
          phone: inviteePhone()
        }
      ]
    }

    const config = {
      headers: { Authorization: `Bearer ${accesstoken()}` }
    };

    axios.post('http://127.0.0.1:4002/api/invite', param, config).then(res => {
      setResponse([...response(), res.data]);
    }).catch(err => {
      setResponse([...response(), err]);
    });
  }

  const loginForm = () => {
    return (
      <div>
        <div>
          <h3>Email :
            <input
              type="text"
              id="email"
              value={email()}
              onChange={(e) => setEmail(e.currentTarget.value)}
            />
          </h3>
        </div>
        <button class={styles.submit} onClick={sendSignup}>Signup</button>
        <div>
          <h3>OTP :
            <input
              type="text"
              id="otp"
              value={otp()}
              onChange={(e) => setOTP(e.currentTarget.value)}
            />
          </h3>
        </div>
        <button class={styles.submit} onClick={verifyUser}>Verify</button>
        <div>{accesstoken()}</div>
        <div>
          <h3>UserId :
            <input
              type="text"
              id="userId"
              value={userId()}
              onChange={(e) => setUserId(e.currentTarget.value)}
            />
          </h3>
        </div>
        <div>
          <h3>FirstName :
            <input
              type="text"
              id="firstName"
              value={firstName()}
              onChange={(e) => setFirstName(e.currentTarget.value)}
            />
          </h3>
        </div>
        <div>
          <h3>LastName :
            <input
              type="text"
              id="lastName"
              value={lastName()}
              onChange={(e) => setLastName(e.currentTarget.value)}
            />
          </h3>
        </div>        
        <button class={styles.submit} onClick={createUser}>Create User</button>
      </div>
    );
  }

  const createWsForm = () => {
    return (
      <div>
        <div>
          <h3>Workspace Name :
            <input
              type="text"
              id="roomName"
              value={roomName()}
              onChange={(e) => setRoomName(e.currentTarget.value)}
            />
          </h3>
        </div>
        <div>
          <h3>Role :
            <input
              type="text"
              id="role"
              value={role()}
              onChange={(e) => setRole(e.currentTarget.value)}
            />
          </h3>
        </div>
        <div>
          <h3>Description :
            <input
              type="text"
              id="desc"
              value={desc()}
              onChange={(e) => setDesc(e.currentTarget.value)}
            />
          </h3>
        </div>
        <div>
          <button class={styles.submit} onClick={createWorkspace}>Create Workspace</button>
          <button class={styles.submit} onClick={getProfile}>Get Profile</button>
          <button class={styles.delete} onClick={deleteUser}>Delete User</button>
        </div>
      </div>
    );
  }

  const contactsForm = () => {
    return (
      <div>
        <div class={styles.container1}>
          <button class={styles.submit} onClick={syncGoogleContacts}>Sync Google Contacts</button>
          <button class={styles.submit} onClick={syncOutlookContacts}>Sync Outlook Contacts</button>
        </div>
        <div>
          <div>
            <h3>Peer User ID :
              <input
                type="text"
                id="peerId"
                value={peerId()}
                onChange={(e) => setPeerId(e.currentTarget.value)}
              />
            </h3>
          </div>
          <div>
            <h3>Workspace ID :
              <input
                type="text"
                id="wsId"
                value={wsId()}
                onChange={(e) => setWsId(e.currentTarget.value)}
              />
            </h3>
          </div>
          <div>
            <h3>Peer Role :
              <input
                type="text"
                id="peerRole"
                value={peerRole()}
                onChange={(e) => setPeerRole(e.currentTarget.value)}
              />
            </h3>
          </div>
        </div>
        <div>
          <button class={styles.submit} onClick={getProfile}>Get Profile</button>
          <button class={styles.submit} onClick={addToWorkspace}>Add to Workspace</button>
          <button class={styles.delete} onClick={deleteUser}>Delete User</button>
        </div>
      </div>
    );
  }

  const inviteForm = () => {
    return (
      <div>
        <div>
          <h3>Enter First Name :
            <input
              type="text"
              id="inviteeFirstName"
              value={inviteeFirstName()}
              onChange={(e) => setInviteeFirstName(e.currentTarget.value)}
            />
          </h3>
        </div>
        <div>
          <h3>Enter Last Name :
            <input
              type="text"
              id="inviteeLastName"
              value={inviteeLastName()}
              onChange={(e) => setInviteeLastName(e.currentTarget.value)}
            />
          </h3>
        </div>
        <div>
          <h3>Enter Email :
            <input
              type="text"
              id="inviteeEmail"
              value={inviteeEmail()}
              onChange={(e) => setInviteeEmail(e.currentTarget.value)}
            />
          </h3>
        </div>
        <div>
          <h3>Enter Phone :
            <input
              type="text"
              id="inviteePhone"
              value={inviteePhone()}
              onChange={(e) => setInviteePhone(e.currentTarget.value)}
            />
          </h3>
        </div>
        <button class={styles.submit} onClick={sendInvite}>Invite</button>
      </div>
    );
  }

  return (
    <div class={styles.container}>
      {login() ?
        createdWs() ? canInvite() ?
          inviteForm() : contactsForm()
          : createWsForm()
        : loginForm()
      }
      <div>
        <h3 className={styles.header}>Log Message</h3>
        <div>
          {response().map((message, index) => {
            return <div key={index} className={styles.response}>
              <h5>{JSON.stringify(message)}</h5>
            </div>
          })}
        </div>
      </div>
    </div>
  );
};

export default App;
