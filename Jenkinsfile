pipeline {
    agent {
        docker {
            image 'docker:dind-alpine'
        }
    }
    stages {
        stage('Initialize') {
            steps {
                script {
                    def dockerHome = tool(name: 'Default', type: 'Docker')
                    env.PATH = "${dockerHome}/bin:${env.PATH}"
                }
            }
        }
        stage('Build Image') {
            steps {
                script {
                    withDockerRegistry(credentialsId: 'nater-registry-creds', 'url': 'https://docker.nater0214.com') [
                        def awbImage = docker.build("docker.nater0214.com/awb:latest")
                        awbImage.push()
                    ]
                }
            }
        }
    }
}
