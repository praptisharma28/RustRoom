// WebRTC Configuration
const rtcConfiguration = {
    iceServers: [
        { urls: 'stun:stun.l.google.com:19302' },
        { urls: 'stun:stun1.l.google.com:19302' }
    ]
};

class WebRTCManager {
    constructor(rustRoom) {
        this.rustRoom = rustRoom;
        this.peerConnections = new Map();
        this.isInitiator = false;
    }

    async createPeerConnection(remoteUserId) {
        const pc = new RTCPeerConnection(rtcConfiguration);
        
        // Add local stream tracks
        if (this.rustRoom.localStream) {
            this.rustRoom.localStream.getTracks().forEach(track => {
                pc.addTrack(track, this.rustRoom.localStream);
            });
        }

        // Handle remote stream
        pc.ontrack = (event) => {
            console.log('Received remote stream from', remoteUserId);
            const remoteVideo = document.getElementById('remoteVideo');
            if (event.streams[0]) {
                remoteVideo.srcObject = event.streams[0];
                this.rustRoom.hideNoStreamMessage();
            }
        };

        // Handle ICE candidates
        pc.onicecandidate = (event) => {
            if (event.candidate) {
                this.rustRoom.send({
                    type: 'WebRTCSignal',
                    target_user: remoteUserId,
                    signal: {
                        type: 'ice-candidate',
                        candidate: event.candidate
                    }
                });
            }
        };

        pc.onconnectionstatechange = () => {
            console.log('Connection state:', pc.connectionState);
            if (pc.connectionState === 'failed' || pc.connectionState === 'disconnected') {
                this.cleanupPeerConnection(remoteUserId);
            }
        };

        this.peerConnections.set(remoteUserId, pc);
        return pc;
    }

    async startCall(remoteUserId) {
        console.log('Starting call with', remoteUserId);
        
        const pc = await this.createPeerConnection(remoteUserId);
        this.isInitiator = true;

        try {
            const offer = await pc.createOffer();
            await pc.setLocalDescription(offer);

            this.rustRoom.send({
                type: 'WebRTCSignal',
                target_user: remoteUserId,
                signal: {
                    type: 'offer',
                    offer: offer
                }
            });
        } catch (error) {
            console.error('Error creating offer:', error);
        }
    }

    async handleSignal(fromUserId, signal) {
        console.log('Handling WebRTC signal from', fromUserId, signal.type);

        let pc = this.peerConnections.get(fromUserId);
        
        if (!pc) {
            pc = await this.createPeerConnection(fromUserId);
        }

        try {
            switch (signal.type) {
                case 'offer':
                    await pc.setRemoteDescription(signal.offer);
                    const answer = await pc.createAnswer();
                    await pc.setLocalDescription(answer);
                    
                    this.rustRoom.send({
                        type: 'WebRTCSignal',
                        target_user: fromUserId,
                        signal: {
                            type: 'answer',
                            answer: answer
                        }
                    });
                    break;

                case 'answer':
                    await pc.setRemoteDescription(signal.answer);
                    break;

                case 'ice-candidate':
                    await pc.addIceCandidate(signal.candidate);
                    break;
            }
        } catch (error) {
            console.error('Error handling WebRTC signal:', error);
        }
    }

    connectToStreamer(streamerId) {
        if (!this.peerConnections.has(streamerId)) {
            console.log('Connecting to streamer:', streamerId);
            this.startCall(streamerId);
        }
    }

    cleanupPeerConnection(userId) {
        const pc = this.peerConnections.get(userId);
        if (pc) {
            pc.close();
            this.peerConnections.delete(userId);
        }
    }

    cleanup() {
        this.peerConnections.forEach((pc, userId) => {
            this.cleanupPeerConnection(userId);
        });
        this.peerConnections.clear();
        this.isInitiator = false;
    }
}
