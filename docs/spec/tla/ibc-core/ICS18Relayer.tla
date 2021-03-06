--------------------------- MODULE ICS18Relayer ----------------------------

(***************************************************************************
 This module contains the specification of a relayer, which is an off-chain 
 process running a relayer algorithm 
 ***************************************************************************)

EXTENDS Integers, FiniteSets, Sequences, IBCCoreDefinitions

CONSTANTS GenerateClientDatagrams, \* toggle generation of client datagrams
          GenerateConnectionDatagrams, \* toggle generation of connection datagrams
          GenerateChannelDatagrams, \* toggle generation of channel datagrams
          GeneratePacketDatagrams \* toggle generation of packet datagrams

ASSUME /\ GenerateClientDatagrams \in BOOLEAN 
       /\ GenerateConnectionDatagrams \in BOOLEAN 
       /\ GenerateChannelDatagrams \in BOOLEAN
       /\ GeneratePacketDatagrams \in BOOLEAN

CONSTANTS MaxHeight, \* set of possible heights of the chains in the system
          MaxVersion, \* maximal connection / channel version (we assume versions are integers)
          MaxPacketSeq  \* maximal packet sequence number
          
VARIABLES chainAstore, \* store of ChainA
          chainBstore, \* store of ChainB
          outgoingDatagrams, \* a function that assigns a set of pending datagrams 
                             \* outgoing from the relayer to each chainID
          outgoingPacketDatagrams, \* a dedicated datagrams channel for packet datagrams 
          relayerHeights, \* a function that assigns a height to each chainID
          closeChannelA, \* flag that triggers closing of the channel end at ChainA
          closeChannelB,  \* flag that triggers closing of the channel end at ChainB
          packetLog \* packet log
          
vars == <<chainAstore, chainBstore, outgoingDatagrams, outgoingPacketDatagrams, relayerHeights, packetLog>>
Heights == 1..MaxHeight \* set of possible heights of the chains in the system
Versions == 1..MaxVersion \* set of possible connection versions                     

GetChainByID(chainID) ==
    IF chainID = "chainA"
    THEN chainAstore
    ELSE chainBstore
    
GetCloseChannelFlag(chainID) ==
    IF chainID = "chainA"
    THEN closeChannelA
    ELSE closeChannelB
 
(***************************************************************************
 Client datagrams
 ***************************************************************************)
\* Compute client datagrams designated for dstChainID. 
\* These are used to update the client for srcChainID on dstChainID.
\* Some client updates might trigger an update of the height that 
\* the relayer stores for srcChainID
ClientDatagrams(srcChainID, dstChainID, relayer) ==
    LET srcChain == GetChainByID(srcChainID) IN
    LET dstChain == GetChainByID(dstChainID) IN
    LET srcChainHeight == GetLatestHeight(srcChain) IN    
    LET srcClientHeight == GetMaxCounterpartyClientHeight(dstChain) IN
    LET srcClientID == GetClientID(srcChainID) IN

    LET emptySetDatagrams == AsSetDatagrams({}) IN

    \* check if the relayer chain height for srcChainID should be updated
    LET srcRelayerChainHeight == 
        IF relayer[srcChainID] < srcChainHeight
        THEN srcChainHeight
        ELSE relayer[srcChainID] IN 
        
    \* create an updated relayer    
    LET updatedRelayer ==
        [relayer EXCEPT ![srcChainID] = srcRelayerChainHeight] IN    
    
    \* generate datagrams for dstChainID
    LET dstDatagrams == 
        IF srcClientHeight = AsInt(nullHeight)
        THEN \* the src client does not exist on dstChainID 
             {AsDatagram([
                type |-> "ClientCreate", 
                height |-> srcChainHeight,
                clientID |-> srcClientID
             ])}
        ELSE \* the src client exists on dstChainID
            IF srcClientHeight < srcChainHeight
            THEN \* the height of the src client on dstChainID is smaller than the height of the src chain
                 {AsDatagram([
                   type |-> "ClientUpdate",
                   height |-> srcChainHeight,
                   clientID |-> srcClientID
                 ])}
            ELSE emptySetDatagrams IN                   
                    
    [datagrams|-> dstDatagrams, relayerUpdate |-> updatedRelayer]    
   
(***************************************************************************
 Connection datagrams
 ***************************************************************************)    
\* Compute connection datagrams designated for dstChainID. 
\* These are used to update the connection end on dstChainID.
ConnectionDatagrams(srcChainID, dstChainID) ==
    LET srcChain == GetChainByID(srcChainID) IN
    LET dstChain == GetChainByID(dstChainID) IN
    
    LET srcConnectionEnd == GetConnectionEnd(srcChain) IN
    LET dstConnectionEnd == GetConnectionEnd(dstChain) IN

    LET srcConnectionID == GetConnectionID(srcChainID) IN
    LET dstConnectionID == GetConnectionID(dstChainID) IN

    LET srcHeight == GetLatestHeight(srcChain) IN
    LET srcClientHeight == GetMaxCounterpartyClientHeight(srcChain) IN
    
    LET emptySetDatagrams == AsSetDatagrams({}) IN

    LET dstDatagrams ==
        IF dstConnectionEnd.state = "UNINIT" /\ srcConnectionEnd.state = "UNINIT" THEN 
            {AsDatagram([
                type |-> "ConnOpenInit", 
                connectionID |-> dstConnectionID, \* "connBtoA" (if srcChainID = "chainA", dstChainID = "chainB")
                clientID |-> GetCounterpartyClientID(dstChainID), \* "clA"
                counterpartyConnectionID |-> srcConnectionID, \* "connAtoB"
                counterpartyClientID |-> GetCounterpartyClientID(srcChainID) \* "clB" 
            ])}
    
        ELSE IF /\ srcClientHeight /= nullHeight
                /\ srcConnectionEnd.state = "INIT" /\ \/ dstConnectionEnd.state = "UNINIT"
                                                      \/ dstConnectionEnd.state = "INIT" THEN 
            {AsDatagram([
                type |-> "ConnOpenTry",
                desiredConnectionID |-> srcConnectionEnd.counterpartyConnectionID, \* "connBtoA" (if srcChainID = "chainA", dstChainID = "chainB")  
                counterpartyConnectionID |-> srcConnectionEnd.connectionID, \* "connAtoB"
                clientID |-> srcConnectionEnd.counterpartyClientID, \* "clA"
                counterpartyClientID |-> srcConnectionEnd.clientID, \* "clB" 
                versions |-> srcConnectionEnd.versions,
                proofHeight |-> srcHeight,
                consensusHeight |-> srcClientHeight
            ])}
         
        ELSE IF /\ srcClientHeight /= nullHeight
                /\ srcConnectionEnd.state = "TRYOPEN" /\ \/ dstConnectionEnd.state = "INIT"
                                                         \/ dstConnectionEnd.state = "TRYOPEN" THEN
            {AsDatagram([
                type |-> "ConnOpenAck",
                connectionID |-> dstConnectionID, \* "connBtoA" (if srcChainID = "chainA", dstChainID = "chainB")
                versions |-> srcConnectionEnd.versions,
                proofHeight |-> srcHeight,
                consensusHeight |-> srcClientHeight
            ])}
         
        ELSE IF srcConnectionEnd.state = "OPEN" /\ dstConnectionEnd.state = "TRYOPEN" THEN
            {AsDatagram([
                type |-> "ConnOpenConfirm",
                connectionID |-> dstConnectionEnd.connectionID, \* "connBtoA" (if srcChainID = "chainA", dstChainID = "chainB")
                proofHeight |-> srcHeight
            ])} 
        ELSE emptySetDatagrams IN

    dstDatagrams
                
(***************************************************************************
 Channel handshake datagrams
 ***************************************************************************)
\* Compute channel datagrams designated for dstChainID. 
\* These are used to update the channel end on dstChainID.
ChannelDatagrams(srcChainID, dstChainID) ==
    LET srcChain == GetChainByID(srcChainID) IN
    LET dstChain == GetChainByID(dstChainID) IN

    LET srcChannelEnd == GetChannelEnd(srcChain) IN
    LET dstChannelEnd == GetChannelEnd(dstChain) IN
    
    LET srcPortID == GetPortID(srcChainID) IN
    LET dstPortID == GetPortID(dstChainID) IN
    
    LET srcChannelID == GetChannelID(srcChainID) IN
    LET dstChannelID == GetChannelID(dstChainID) IN

    LET srcHeight == GetLatestHeight(srcChain) IN
    
    LET emptySetDatagrams == AsSetDatagrams({}) IN

    LET dstDatagrams ==
        IF dstChannelEnd.state = "UNINIT" /\ srcChannelEnd.state = "UNINIT" THEN 
            {AsDatagram([
                type |-> "ChanOpenInit", 
                portID |-> dstPortID, \* "portB" (if srcChainID = "chainA", dstChainID = "chainB")
                channelID |-> dstChannelID, \* "chanBtoA" 
                counterpartyPortID |-> srcPortID, \* "portA"
                counterpartyChannelID |-> srcChannelID \* "chanAtoB" 
            ])}
    
        ELSE IF srcChannelEnd.state = "INIT" /\ \/ dstChannelEnd.state = "UNINIT"
                                                \/ dstChannelEnd.state = "INIT" THEN 
            {AsDatagram([
                type |-> "ChanOpenTry",
                portID |-> dstPortID, \* "portB" (if srcChainID = "chainA", dstChainID = "chainB")
                channelID |-> dstChannelID, \* "chanBtoA"   
                counterpartyPortID |-> srcPortID, \* "portA"
                counterpartyChannelID |-> srcChannelID, \* "chanAtoB"
                proofHeight |-> srcHeight
            ])} 
         
        ELSE IF srcChannelEnd.state = "TRYOPEN" /\ \/ dstChannelEnd.state = "INIT"
                                                   \/ dstChannelEnd.state = "TRYOPEN" THEN
            {AsDatagram([
                type |-> "ChanOpenAck",
                portID |-> dstChannelEnd.portID, \* "portB" (if srcChainID = "chainA", dstChainID = "chainB")
                channelID |-> dstChannelEnd.channelID, \* "chanBtoA"
                proofHeight |-> srcHeight
            ])} 
         
        ELSE IF srcChannelEnd.state = "OPEN" /\ dstChannelEnd.state = "TRYOPEN" THEN
            {AsDatagram([
                type |-> "ChanOpenConfirm",
                portID |-> dstChannelEnd.portID, \* "portB" (if srcChainID = "chainA", dstChainID = "chainB")
                channelID |-> dstChannelEnd.channelID, \* "chanBtoA"
                proofHeight |-> srcHeight
            ])} 
    
        \* channel closing datagrams creation only for open channels
        ELSE IF dstChannelEnd.state = "OPEN" /\ GetCloseChannelFlag(dstChannelID) THEN
            {AsDatagram([
                type |-> "ChanCloseInit", 
                portID |-> dstChannelEnd.portID, \* "portB" (if srcChainID = "chainA", dstChainID = "chainB")
                channelID |-> dstChannelEnd.channelID \* "chanBtoA"   
            ])}
           
        ELSE IF /\ srcChannelEnd.state = "CLOSED" 
                /\ dstChannelEnd.state /= "CLOSED" 
                /\ dstChannelEnd.state /= "UNINIT" THEN 
            {AsDatagram([
                type |-> "ChanCloseConfirm", 
                portID |-> dstChannelEnd.portID, \* "portB" (if srcChainID = "chainA", dstChainID = "chainB")
                channelID |-> dstChannelEnd.channelID, \* "chanBtoA"   
                proofHeight |-> srcHeight
            ])}
           
        ELSE emptySetDatagrams IN

    dstDatagrams

(***************************************************************************
 Packet datagrams
 ***************************************************************************)
\* Compute a packet datagram based on the packetLogEntry
PacketDatagram(packetLogEntry) ==
    \* get chainID and its channel end
    LET chainID == packetLogEntry.srcChainID IN
    LET channelEnd == GetChainByID(chainID).connectionEnd.channelEnd IN
    
    \* get portID and counterpartyPortID
    LET portID == channelEnd.portID IN \* "portA" (if srcChainID = "chainA")
    LET counterpartyPortID == channelEnd.counterpartyPortID IN \* "portB" (if srcChainID = "chainA")
    
    \* get channelID and counterpartyChannelID
    LET channelID == channelEnd.channelID IN \* "chanAtoB" (if srcChainID = "chainA")
    LET counterpartyChannelID == channelEnd.counterpartyChannelID IN \* "chanBtoA" (if srcChainID = "chainA")
    
    LET srcHeight == GetLatestHeight(GetChainByID(chainID)) IN
    
    \* the srcChannelID of the packet that is received is channelID,
    \* the dstChannelID of the packet that is received is counterpartyChannelID
    LET recvPacket(logEntry) == AsPacket([sequence |-> logEntry.sequence, 
                                 timeoutHeight |-> logEntry.timeoutHeight,
                                 srcPortID |-> portID,
                                 srcChannelID |-> channelID,
                                 dstPortID |-> counterpartyPortID,
                                 dstChannelID |-> counterpartyChannelID]) IN
    
    \* the srcChannelID of the packet that is acknowledged is counterpartyChannelID,
    \* the dstChannelID of the packet that is acknowledged is channelID
    LET ackPacket(logEntry) == AsPacket([sequence |-> logEntry.sequence, 
                                 timeoutHeight |-> logEntry.timeoutHeight,
                                 srcPortID |-> counterpartyPortID,
                                 srcChannelID |-> counterpartyChannelID,
                                 dstPortID |-> portID,
                                 dstChannelID |-> channelID]) IN
    
    IF packetLogEntry.type = "PacketSent"
    THEN AsDatagram([type |-> "PacketRecv",
          packet |-> recvPacket(packetLogEntry),  
          proofHeight |-> srcHeight])
    ELSE IF packetLogEntry.type = "WriteAck"
         THEN AsDatagram([type |-> "PacketAck",
                  packet |-> ackPacket(packetLogEntry),
                  acknowledgement |-> packetLogEntry.acknowledgement,  
                  proofHeight |-> srcHeight])
         ELSE NullDatagram 

(***************************************************************************
 Compute client, connection, channel datagrams (from srcChainID to dstChainID)
 ***************************************************************************)
\* Currently supporting:
\*  -  ICS 02: Client updates
\*  -  ICS 03: Connection handshake
\*  -  ICS 04: Channel handshake 
ComputeDatagrams(srcChainID, dstChainID) ==
    \* ICS 02 : Clients
    \* - Determine if light clients needs to be updated 
    LET clientDatagrams == 
        IF GenerateClientDatagrams 
        THEN ClientDatagrams(srcChainID, dstChainID, relayerHeights) 
        ELSE [datagrams |-> AsSetDatagrams({}), relayerUpdate |-> relayerHeights] IN
    
    \* ICS 03 : Connections
    \* - Determine if any connection handshakes are in progress
    LET connectionDatagrams == 
        IF GenerateConnectionDatagrams
        THEN ConnectionDatagrams(srcChainID, dstChainID)
        ELSE AsSetDatagrams({}) IN
    
    \* ICS 04 : Channels & Packets
    \* - Determine if any channel handshakes are in progress
    LET channelDatagrams == 
        IF GenerateChannelDatagrams 
        THEN ChannelDatagrams(srcChainID, dstChainID)
        ELSE AsSetDatagrams({}) IN

    [datagrams |-> clientDatagrams.datagrams \union 
                   connectionDatagrams \union 
                   channelDatagrams, 
     relayerUpdate |-> clientDatagrams.relayerUpdate]

(***************************************************************************
 Relayer actions
 ***************************************************************************)   
\* Update the height of the relayer client for some chainID
UpdateRelayerClientHeight(chainID) ==
    LET chainLatestHeight == GetLatestHeight(GetChainByID(chainID)) IN
    /\ relayerHeights[chainID] < chainLatestHeight
    /\ relayerHeights' = [relayerHeights EXCEPT 
                            ![chainID] = GetLatestHeight(GetChainByID(chainID))
                         ]
    /\ UNCHANGED <<chainAstore, chainBstore, outgoingDatagrams>>  
    /\ UNCHANGED <<outgoingPacketDatagrams, packetLog>>

\* for two chains, srcChainID and dstChainID, where srcChainID /= dstChainID, 
\* create the pending datagrams and update the corresponding sets of pending datagrams
Relay(srcChainID, dstChainID) ==
    LET datagramsAndRelayerUpdate == ComputeDatagrams(srcChainID, dstChainID) IN
    /\ srcChainID /= dstChainID
    /\ outgoingDatagrams' = 
            [outgoingDatagrams EXCEPT 
                ![dstChainID] = AsSetDatagrams(outgoingDatagrams[dstChainID] 
                                \union 
                                datagramsAndRelayerUpdate.datagrams)
            ]
    /\ relayerHeights' = datagramsAndRelayerUpdate.relayerUpdate       
    /\ UNCHANGED <<chainAstore, chainBstore>>
    /\ UNCHANGED <<outgoingPacketDatagrams, packetLog>>

\* given an entry from the packet log, create a packet datagram and 
\* append it to the outgoing packet datagram queue for dstChainID      
RelayPacketDatagram(srcChainID, dstChainID) ==
    /\ packetLog /= AsPacketLog(<<>>)
    /\ GeneratePacketDatagrams
    /\ LET packetLogEntry == Head(packetLog) IN
       LET packetDatagram == PacketDatagram(packetLogEntry) IN
       \* if srcChainID matches the one from the log entry
       /\ packetLogEntry.srcChainID = srcChainID
       \* if dstChainID is the counterparty chain of srcChainID
       /\ dstChainID = GetCounterpartyChainID(packetLogEntry.srcChainID)
       /\ packetDatagram /= NullDatagram
       /\ outgoingPacketDatagrams' = [outgoingPacketDatagrams EXCEPT 
            ![dstChainID] = Append(outgoingPacketDatagrams[dstChainID], 
                                   AsPacketDatagram(packetDatagram))]
       /\ packetLog' = Tail(packetLog)
       /\ UNCHANGED <<chainAstore, chainBstore>>
       /\ UNCHANGED <<outgoingDatagrams, relayerHeights>>
    
\*    
\*    \* get dstChainID
\*    LET dstChainID == GetCounterpartyChainID(packetLogEntry.srcChainID) IN
\*    \* create a packet datagram from packet log entry
\*    LET packetDatagram == PacketDatagram(packetLogEntry) IN 
\*    
\*    IF packetDatagram /= NullDatagram
\*    THEN [outgoingPacketDatagrams EXCEPT 
\*            ![dstChainID] = Append(outgoingPacketDatagrams[dstChainID], 
\*                                   AsPacketDatagram(packetDatagram))]
\*    ELSE outgoingPacketDatagrams      

\* update the relayer client heights
UpdateClient ==
    \E chainID \in ChainIDs : UpdateRelayerClientHeight(chainID)
    
\* create client, connection, channel, packet datagrams    
CreateDatagrams ==
    \E srcChainID \in ChainIDs : \E dstChainID \in ChainIDs : 
        \* client, connection, channel datagrams
        Relay(srcChainID, dstChainID)
        
\*        /\ \/ /\ packetLog /= AsPacketLog(<<>>)
\*              /\ Head(packetLog).srcChainID = srcChainID
\*              /\ GeneratePacketDatagrams
\*              \* packet datagrams   
\*              /\ outgoingPacketDatagrams' = RelayPacketDatagram(AsPacketLogEntry(Head(packetLog)))
\*              /\ packetLog' = Tail(packetLog)
\*           \/ /\ UNCHANGED <<outgoingPacketDatagrams, packetLog>>

CreatePacketDatagrams ==
    \E srcChainID \in ChainIDs : \E dstChainID \in ChainIDs :
        RelayPacketDatagram(srcChainID, dstChainID)
    
    

(***************************************************************************
 Specification
 ***************************************************************************)
\* Initial state predicate
\*    Initially:
\*        - the relayer heights are uninitialized (i.e., their height is nullHeight)
\*        - there are no datagrams
Init == 
    /\ relayerHeights = [chainID \in ChainIDs |-> AsInt(nullHeight)]
    /\ outgoingDatagrams = [chainID \in ChainIDs |-> AsSetDatagrams({})]
    /\ outgoingPacketDatagrams = [chainID \in ChainIDs |-> AsSeqPacketDatagrams(<<>>)] 
    
\* Next state action
\*    The relayer either:
\*        - updates its clients, or
\*        - creates datagrams, or 
\*        - does nothing
Next ==
    \/ UpdateClient
    \/ CreateDatagrams
    \/ CreatePacketDatagrams
    \/ UNCHANGED vars    
       
\* Fairness constraints
Fairness ==
\*    WF_vars(Next)
\*    /\ WF_vars(CreateDatagrams)
\*    /\ WF_vars(CreatePacketDatagrams)
    /\ \A chainID \in ChainIDs : 
            WF_vars(UpdateRelayerClientHeight(chainID))
    /\ \A srcChainID \in ChainIDs : \A dstChainID \in ChainIDs : 
            WF_vars(Relay(srcChainID, dstChainID))
    /\ \A srcChainID \in ChainIDs : \A dstChainID \in ChainIDs :
            WF_vars(RelayPacketDatagram(srcChainID, dstChainID))
            
    
               
(***************************************************************************
 Invariants
 ***************************************************************************)
\* Type invariant
TypeOK ==
    /\ relayerHeights \in [ChainIDs -> Heights \union {nullHeight}]
    /\ outgoingDatagrams \in [ChainIDs -> SUBSET Datagrams(Heights, MaxPacketSeq, Versions)]
    /\ outgoingPacketDatagrams \in [ChainIDs -> Seq(Datagrams(Heights, MaxPacketSeq, Versions))]

=============================================================================
\* Modification History
\* Last modified Tue Feb 02 09:58:43 CET 2021 by ilinastoilkovska
\* Created Fri Mar 06 09:23:12 CET 2020 by ilinastoilkovska
