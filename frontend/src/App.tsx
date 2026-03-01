import 'bootstrap/dist/css/bootstrap.min.css';
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { Container, Alert } from 'react-bootstrap';
import { useState, useEffect } from 'react';
import Layout from './components/Layout';
import OnboardingPage from './pages/OnboardingPage';
import ProjectsPage from './pages/ProjectsPage';
import ProjectDetailPage from './pages/ProjectDetailPage';
import apiClient from './api/client';

function App() {
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [authError, setAuthError] = useState<string | null>(null);

  useEffect(() => {
    // Check if we have a token in localStorage
    const token = localStorage.getItem('kroma_api_token');
    if (token) {
      apiClient.setToken(token);
      setIsAuthenticated(true);
    }
  }, []);

  const handleAuthComplete = (token: string) => {
    localStorage.setItem('kroma_api_token', token);
    apiClient.setToken(token);
    setIsAuthenticated(true);
    setAuthError(null);
  };

  const handleLogout = () => {
    localStorage.removeItem('kroma_api_token');
    apiClient.setToken(null);
    setIsAuthenticated(false);
  };

  // Show onboarding if not authenticated
  if (!isAuthenticated) {
    return (
      <Container className="py-5">
        <OnboardingPage onAuthComplete={handleAuthComplete} onError={setAuthError} />
        {authError && (
          <Alert variant="danger" className="mt-3">
            {authError}
          </Alert>
        )}
      </Container>
    );
  }

  return (
    <BrowserRouter>
      <Layout onLogout={handleLogout}>
        <Routes>
          <Route path="/" element={<Navigate to="/projects" replace />} />
          <Route path="/projects" element={<ProjectsPage />} />
          <Route path="/projects/:slug" element={<ProjectDetailPage />} />
        </Routes>
      </Layout>
    </BrowserRouter>
  );
}

export default App;
