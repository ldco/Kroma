import React, { useState, useEffect } from 'react';
import { Card, Button, Form, Modal, Spinner, Alert, Row, Col, Badge } from 'react-bootstrap';
import { Link } from 'react-router-dom';
import { Plus, Folder, Calendar } from 'react-bootstrap-icons';
import apiClient, { type Project, type CreateProjectRequest } from '../api/client';

const ProjectsPage: React.FC = () => {
  const [projects, setProjects] = useState<Project[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [isCreating, setIsCreating] = useState(false);
  const [formData, setFormData] = useState<CreateProjectRequest>({
    name: '',
    slug: '',
    description: '',
  });

  useEffect(() => {
    loadProjects();
  }, []);

  const loadProjects = async () => {
    setIsLoading(true);
    setError(null);
    try {
      const response = await apiClient.listProjects();
      if (response.ok) {
        setProjects(response.data?.projects || []);
      } else {
        setError(response.error || 'Failed to load projects');
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to load projects';
      setError(message);
    } finally {
      setIsLoading(false);
    }
  };

  const handleCreateProject = async () => {
    if (!formData.name.trim()) {
      setError('Project name is required');
      return;
    }

    setIsCreating(true);
    setError(null);
    try {
      const payload: CreateProjectRequest = {
        name: formData.name.trim(),
        slug: formData.slug?.trim(),
        description: formData.description?.trim(),
      };

      const response = await apiClient.createProject(payload);
      if (response.ok && response.data) {
        setShowCreateModal(false);
        setFormData({ name: '', slug: '', description: '' });
        loadProjects();
      } else {
        setError(response.error || 'Failed to create project');
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to create project';
      setError(message);
    } finally {
      setIsCreating(false);
    }
  };

  const handleModalClose = () => {
    setShowCreateModal(false);
    setFormData({ name: '', slug: '', description: '' });
    setError(null);
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
    });
  };

  if (isLoading) {
    return (
      <div className="text-center py-5">
        <Spinner animation="border" variant="primary" />
        <p className="mt-3 text-muted">Loading projects...</p>
      </div>
    );
  }

  return (
    <>
      <div className="d-flex justify-content-between align-items-center mb-4">
        <h1>Projects</h1>
        <Button variant="primary" onClick={() => setShowCreateModal(true)}>
          <Plus className="me-2" />
          New Project
        </Button>
      </div>

      {error && (
        <Alert variant="danger" onClose={() => setError(null)} dismissible>
          {error}
        </Alert>
      )}

      {projects.length === 0 ? (
        <Card className="text-center py-5">
          <Card.Body>
            <Folder size={48} className="text-muted mb-3" />
            <h4>No projects yet</h4>
            <p className="text-muted mb-4">
              Create your first project to start building your comic universe
            </p>
            <Button variant="primary" onClick={() => setShowCreateModal(true)}>
              <Plus className="me-2" />
              Create Project
            </Button>
          </Card.Body>
        </Card>
      ) : (
        <Row xs={1} md={2} lg={3} className="g-4">
          {projects.map((project) => (
            <Col key={project.id}>
              <Card className="h-100">
                <Card.Body>
                  <div className="d-flex justify-content-between align-items-start mb-2">
                    <Card.Title className="mb-0">{project.name}</Card.Title>
                    {project.status === 'archived' && (
                      <Badge bg="secondary">Archived</Badge>
                    )}
                  </div>
                  {project.description && (
                    <Card.Text className="text-muted small">
                      {project.description}
                    </Card.Text>
                  )}
                  <div className="d-flex justify-content-between align-items-center mt-3">
                    <small className="text-muted">
                      <Calendar className="me-1" size={12} />
                      {formatDate(project.created_at)}
                    </small>
                    <Button
                      as={Link as any}
                      to={`/projects/${project.slug}`}
                      variant="outline-primary"
                      size="sm"
                    >
                      Open
                    </Button>
                  </div>
                </Card.Body>
              </Card>
            </Col>
          ))}
        </Row>
      )}

      {/* Create Project Modal */}
      <Modal show={showCreateModal} onHide={handleModalClose}>
        <Modal.Header closeButton>
          <Modal.Title>Create New Project</Modal.Title>
        </Modal.Header>
        <Modal.Body>
          <Form>
            <Form.Group className="mb-3">
              <Form.Label>Project Name *</Form.Label>
              <Form.Control
                type="text"
                placeholder="My Comic Universe"
                value={formData.name}
                onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                autoFocus
              />
              <Form.Text className="text-muted">
                A descriptive name for your story universe
              </Form.Text>
            </Form.Group>

            <Form.Group className="mb-3">
              <Form.Label>Slug (optional)</Form.Label>
              <Form.Control
                type="text"
                placeholder="my-comic-universe"
                value={formData.slug}
                onChange={(e) => setFormData({ ...formData, slug: e.target.value })}
              />
              <Form.Text className="text-muted">
                URL-friendly identifier (auto-generated if not provided)
              </Form.Text>
            </Form.Group>

            <Form.Group className="mb-3">
              <Form.Label>Description (optional)</Form.Label>
              <Form.Control
                as="textarea"
                rows={3}
                placeholder="A brief description of your project..."
                value={formData.description}
                onChange={(e) => setFormData({ ...formData, description: e.target.value })}
              />
            </Form.Group>
          </Form>

          {error && (
            <Alert variant="danger" className="mb-0" onClose={() => setError(null)} dismissible>
              {error}
            </Alert>
          )}
        </Modal.Body>
        <Modal.Footer>
          <Button variant="secondary" onClick={handleModalClose}>
            Cancel
          </Button>
          <Button
            variant="primary"
            onClick={handleCreateProject}
            disabled={isCreating || !formData.name.trim()}
          >
            {isCreating ? (
              <>
                <Spinner as="span" animation="border" size="sm" className="me-2" />
                Creating...
              </>
            ) : (
              'Create Project'
            )}
          </Button>
        </Modal.Footer>
      </Modal>
    </>
  );
};

export default ProjectsPage;
