package main

import (
	"context"
	"log"
	"os"
	"time"

	v2 "github.com/cloudevents/sdk-go/v2"
	"github.com/cloudevents/sdk-go/v2/event"
	"github.com/cloudevents/sdk-go/v2/protocol"
	"github.com/cloudevents/sdk-go/v2/protocol/http"
	"github.com/kelseyhightower/envconfig"
	"github.com/knix-microfunctions/workflowmesh/event-performance/my"
)

/*
Base implementation of a function worker that would

- receive and acknowledge an event
- depending on the function required, place it in a channel waiting to be processed.
- Functions are go routines sequentially processing events (TODO: concurrency)
- When finished, the function tries to send the request to the next function along the flow
*/

type envConfig struct {
	DataDir string `envconfig:"DATADIR"`
	Type    string `envconfig:"TYPE"`
}

const (
	MaxIdleConnections int = 20
	RequestTimeout     int = 5
)

var (
	env envConfig
)

func gotEvent(inputEvent event.Event) (*event.Event, protocol.Result) {
	// before processing the reques, add entryTime
	entryTime := time.Now().UnixNano()
	data := &my.Event{}
	if err := inputEvent.DataAs(data); err != nil {
		log.Printf("Got error while unmarshalling data: %s", err.Error())
		return nil, http.NewResult(400, "got error while unmarshalling data: %w", err)
	}

	// log.Printf("Received a new event [%d]: ", entryTime)
	// log.Printf("[%v] %s %s: %+v", inputEvent.Time(), inputEvent.Source(), inputEvent.Type(), data)

	// append time to the data
	data.Times = append(data.Times, entryTime)

	// Do some work on the event
	/*
		for i := 100000000; i > 0; i-- {
			i = i - rand.Intn(10)
		}
	*/

	// Create output event
	outputEvent := inputEvent.Clone()

	// Resolve type
	if env.Type != "" {
		outputEvent.SetType(env.Type)
	}

	data.Times = append(data.Times, time.Now().UnixNano())
	log.Println("Event done", data.Times)
	outputEvent.SetData(event.ApplicationJSON, data)

	// log.Println("Transform the event to: ")
	// log.Printf("[%s] %s %s: %+v", outputEvent.Time(), outputEvent.Source(), outputEvent.Type(), data)

	return &outputEvent, nil

}

func main() {
	if err := envconfig.Process("", &env); err != nil {
		log.Printf("[ERROR] Failed to process env var: %s", err)
		os.Exit(1)
	}

	/*
		// HTTP Server
		mux := http.NewServeMux()
		mux.Handle("/data", http.FileServer(http.Dir(env.DataDir)))
		httpServer := &http.Server{
			Addr:    "0.0.0.0:7070",
			Handler: mux,
		}

		quit := make(chan os.Signal, 1)
		signal.Notify(quit, os.Interrupt)
		signal.Notify(quit, syscall.SIGTERM)
		done := make(chan bool, 1)

		// Shutdown hook
		go func() {
			<-quit // wait on signal
			log.Println("Server is shutting down...")

			ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
			defer cancel()
			httpServer.SetKeepAlivesEnabled(false)
			if err := httpServer.Shutdown(ctx); err != nil {
				log.Fatalf("Could not gracefully shutdown the server: %v\n", err)
			}
			close(done)
			close(quit)
		}()

		go func() {
			log.Printf("listening on 7070 to serve data")
			if err := httpServer.ListenAndServe(); err != nil && err != http.ErrServerClosed {
				fmt.Printf("ListenAndServe Error %+v", err)
			}
			<-done
		}()
	*/
	c, err := v2.NewDefaultClient()
	if err != nil {
		log.Fatalf("failed to create client, %v", err)
	}
	log.Printf("listening on 8080 to respond to events")
	log.Fatalf("failed to start receiver: %s", c.StartReceiver(context.Background(), gotEvent))
}
