### Source code for `frontend` running inside each workflow container

The `frontend` receives the user requests for workflow invocation from outside the container. It invokes `service1` and then waits for the last `service` (`service2`) to provide a response of workflow invocation back to the `frontend`. `frontend` then responds to the users' workflow invocation. 
