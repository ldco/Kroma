import React, { useState, useEffect } from 'react';
import { useParams } from 'react-router-dom';
import { Card, Spinner, Alert, Tabs, Tab, Button } from 'react-bootstrap';
import { ArrowLeft } from 'react-bootstrap-icons';
import apiClient, { type Project } from '../api/client';

const ProjectDetailPage: React.FC = () => {
  const { slug } = useParams<{ slug: string }>();
  const [project, setProject] = useState<Project | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (slug) {
      loadProject(slug);
    }
  }, [slug]);

  const loadProject = async (projectSlug: string) => {
    setIsLoading(true);
    setError(null);
    try {
      const response = await apiClient.getProject(projectSlug);
      if (response.ok && response.data) {
        setProject(response.data.project);
      } else {
        setError(response.error || 'Failed to load project');
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to load project';
      setError(message);
    } finally {
      setIsLoading(false);
    }
  };

  if (isLoading) {
    return (
      <div className="text-center py-5">
        <Spinner animation="border" variant="primary" />
        <p className="mt-3 text-muted">Loading project...</p>
      </div>
    );
  }

  if (error || !project) {
    return (
      <Alert variant="danger">
        {error || 'Project not found'}
      </Alert>
    );
  }

  return (
    <>
      <div className="mb-4">
        <Button variant="link" className="ps-0" onClick={() => window.history.back()}>
          <ArrowLeft className="me-1" />
          Back to Projects
        </Button>
      </div>

      <Card className="mb-4">
        <Card.Body>
          <div className="d-flex justify-content-between align-items-start">
            <div>
              <h1 className="mb-2">{project.name}</h1>
              {project.description && (
                <p className="text-muted mb-0">{project.description}</p>
              )}
            </div>
            <div className="text-end">
              <small className="text-muted">
                Created: {new Date(project.created_at).toLocaleDateString()}
              </small>
            </div>
          </div>
        </Card.Body>
      </Card>

      <Tabs defaultActiveKey="overview" className="mb-4">
        <Tab eventKey="overview" title="Overview">
          <Card>
            <Card.Body>
              <h5>Project Overview</h5>
              <p className="text-muted">
                This is your project universe. Here you'll manage style guides,
                character references, and generate images with consistent identity.
              </p>
              <div className="row mt-4">
                <div className="col-md-3">
                  <Card className="bg-light">
                    <Card.Body className="text-center">
                      <h3 className="mb-0">0</h3>
                      <small className="text-muted">Style Guides</small>
                    </Card.Body>
                  </Card>
                </div>
                <div className="col-md-3">
                  <Card className="bg-light">
                    <Card.Body className="text-center">
                      <h3 className="mb-0">0</h3>
                      <small className="text-muted">Characters</small>
                    </Card.Body>
                  </Card>
                </div>
                <div className="col-md-3">
                  <Card className="bg-light">
                    <Card.Body className="text-center">
                      <h3 className="mb-0">0</h3>
                      <small className="text-muted">Reference Sets</small>
                    </Card.Body>
                  </Card>
                </div>
                <div className="col-md-3">
                  <Card className="bg-light">
                    <Card.Body className="text-center">
                      <h3 className="mb-0">0</h3>
                      <small className="text-muted">Runs</small>
                    </Card.Body>
                  </Card>
                </div>
              </div>
            </Card.Body>
          </Card>
        </Tab>
        <Tab eventKey="settings" title="Settings">
          <Card>
            <Card.Body>
              <h5>Project Settings</h5>
              <p className="text-muted">
                Configure storage, providers, and project-specific settings.
              </p>
              <div className="mt-3">
                <strong>Slug:</strong> <code>{project.slug}</code>
              </div>
              <div className="mt-2">
                <strong>Status:</strong> <span className="text-capitalize">{project.status}</span>
              </div>
            </Card.Body>
          </Card>
        </Tab>
      </Tabs>
    </>
  );
};

export default ProjectDetailPage;
