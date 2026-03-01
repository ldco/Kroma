import React, { useState } from 'react';
import { Card, Button, Spinner } from 'react-bootstrap';
import apiClient from '../api/client';

interface OnboardingPageProps {
  onAuthComplete: (token: string) => void;
  onError: (error: string) => void;
}

const OnboardingPage: React.FC<OnboardingPageProps> = ({ onAuthComplete, onError }) => {
  const [isLoading, setIsLoading] = useState(false);

  const handleGetStarted = async () => {
    setIsLoading(true);
    try {
      const response = await apiClient.bootstrapToken();
      if (response.ok && response.data?.token) {
        onAuthComplete(response.data.token);
      } else {
        onError(response.error || 'Failed to authenticate');
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Authentication failed';
      onError(message);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="min-vh-100 d-flex align-items-center justify-content-center">
      <Card className="shadow-lg" style={{ maxWidth: '500px', width: '100%' }}>
        <Card.Body className="p-5">
          <div className="text-center mb-4">
            <img src="/logo.png" alt="Kroma" height="80" className="mb-3" />
            <h1 className="h2 mb-2">Welcome to Kroma</h1>
            <p className="text-muted">
              Project-first comic and graphic novel production with AI-powered workflows
            </p>
          </div>

          <Card className="bg-light mb-4">
            <Card.Body>
              <h5 className="h6 mb-3">What you can do with Kroma:</h5>
              <ul className="mb-0">
                <li>Create isolated project universes with consistent style</li>
                <li>Maintain stable character identities across many images</li>
                <li>Generate images with staged workflows (style → time → weather)</li>
                <li>Post-process with upscale, color correction, and background removal</li>
                <li>Export production-ready assets with reproducibility metadata</li>
              </ul>
            </Card.Body>
          </Card>

          <Button
            variant="primary"
            size="lg"
            className="w-100"
            onClick={handleGetStarted}
            disabled={isLoading}
          >
            {isLoading ? (
              <>
                <Spinner as="span" animation="border" size="sm" className="me-2" />
                Getting Started...
              </>
            ) : (
              'Get Started'
            )}
          </Button>

          <p className="text-muted text-center mt-3 mb-0 small">
            This will create your first API token for local development
          </p>
        </Card.Body>
      </Card>
    </div>
  );
};

export default OnboardingPage;
