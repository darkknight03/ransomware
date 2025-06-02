- server should store open connections
- ability to login to server
    - once logged in, send commands
    - such as view connections, send message to client, upload file to client, receive message from client
- multi-threading on server for handling clients/requests
    - thread pool, fork/join model, the single-threaded async I/O model, or the multi-threaded async I/O model
- handle sig-term gracefully
- logging


// Add functionality to listen and send messages to client at same time
// Add HashMap to store client connections with their addresses and other info
// if client has not checked for a certain amount of time, remove from client connections HashMap and end connection

1. Agent Communication Protocol:
- How agents connect: Define how agents will "call home" to the C2. Options include HTTP/S. DNS, TCP sockets, or ICMP (less reliable). HTTP/S often preferred for its ability to blend with normal traffic.
- Encryption: Use (TLS, others)
- Check-in Mechanism: beaconing with jitter to avoid predicatable network patterns
- Data Format: how data (tasks, results) is structured (JSON, XML, custom binary format). Needs to be consistent between C2 and agent

2. Agent Management
- Registration/Identification: assign unique IDs to agents when they first connect
- Basic tracking: keep track of active vs inactive agents, last seen time, basic host info (IP, hostname, OS, etc.)
- List Agents: command to show connected/known agents and their status
- Select Agent: command to "focus" on a specific agent for tasking/querying info

3. Tasking System
- Send Command: Ability to send a basic command (shell command) to a selected agent
- Receive Output: Mechanism for the agent to send the result/output of command back to C2
- Task Queue (per agent): Store tasks for an agent if it's not currently connected, delivering them upon next check-in

4. Logging
- C2 Activity Log: Record operator commands, agent connections, errors, etc. Essential for debugging and analysis
- Agent Output Storage: store results from agents in csv/database

5. Security
- Operator Authentication: secure login for accessing the C2 interface
- Input validation: sanitize inputs received from agents and the operator interface to prevent injection attacks against your C2


Advanced Features
- Multiple Communication Channels: Support for agents communicating over different protocols (e.g., fallback from HTTPS to DNS if HTTPS is blocked)
- Encoding/obfuscation: Add layers of encoding or simple obfuscation to potentially hinder signature-based detection
- Visual interface: web-based GUI for managing agents and viewing data
- File Transfer: upload and download files to and from agents
- Interactive shell: provide pseudo-interactive shell session with an agent
- Buil-in Modules: common post-exploitation tasks as modules (screenshot, list processes, dump credentials, run scripts)
- Structured storage: database for staring agent info, logs, and results
- Proxies: ability to configure agents to connect through intermediary servers (redirectors) to obfuscate true C2 location
- Rate limiting: Protect the C2 from accidental DoS

Key Design Considerations:
- Modularity
- Scalability
- Error Handling
- Configuration: make settings like C2 address, ports, keys, etc easily configurable

