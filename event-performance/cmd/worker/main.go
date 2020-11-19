package main

import (
	"context"
	"flag"
	"fmt"
	"log"
	"os"
	"time"

	cloudevents "github.com/cloudevents/sdk-go/v2"
	"github.com/cloudevents/sdk-go/v2/event"
	"github.com/cloudevents/sdk-go/v2/protocol"
	cehttp "github.com/cloudevents/sdk-go/v2/protocol/http"
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

const (
	MaxIdleConnections int = 20
	RequestTimeout     int = 5
	NumFunctions       int = 5
)

type envConfig struct {
	DataDir string `envconfig:"DATADIR"`
	Sink    string `envconfig:"K_SINK"`
}

var (
	chEvent   [NumFunctions]chan my.Event
	eventType string
	sink      string
)

func init() {
	flag.StringVar(&eventType, "eventType", "dev.knative.eventing.samples.heartbeat", "the event-type (CloudEvents)")
	flag.StringVar(&sink, "sink", "", "the host url to heartbeat to")
}

/*
func handleRequest(w http.ResponseWriter, r *http.Request) {

	// before processing the reques, add entryTime
	entryTime := time.Now().UnixNano()

	event := &event.Event{}
	body, err := ioutil.ReadAll(r.Body)
	if err != nil {
		log.Println("handler: Error reading body", err)
		http.Error(w, "can't read body", http.StatusBadRequest)
		return
	}
	err = json.Unmarshal([]byte(body), &event)
	if err != nil {
		log.Println("handler: Error parsing json", err)
		http.Error(w, "can't parse json", http.StatusBadRequest)
		return
	}

	// append eventMsgAppender to message of the event
	event.Times = append(event.Times, entryTime)

	// we're done, nowhere else to send
	if env.Sink == "" {
		//b := make([]string, len(event.Times))
		//for i,t := range event.Times {
		//  b[i] = strconv.Itoa(t)
		//	event.Times[i] - event.Times[i-1]
		//}
		//log.Println("Event done", strings.Join(b, ","))
		log.Println("Event done", event.Times)
		w.WriteHeader(http.StatusOK)
		return
	}

	// parse int from subject
	var functionNum int
	path := strings.Split(r.URL.Path, "/")
	_, err = fmt.Sscanf(path[len(path)-1], "function%d", &functionNum)
	if err != nil {
		log.Println("Can't parse function number from path segment", path[len(path)-1])
		w.WriteHeader(http.StatusNotFound)
		return
	}
	if functionNum > len(chEvent) {
		log.Printf("Requested function number %d, but only got %d functions", functionNum, len(chEvent))
		w.WriteHeader(http.StatusNotFound)
		return
	}
	chEvent[functionNum-1] <- *event
	w.WriteHeader(http.StatusOK)
	return
}
*/
func handleEvent(inputEvent event.Event) protocol.Result {

	// before processing the reques, add entryTime
	entryTime := time.Now().UnixNano()
	data := &my.Event{}
	if err := inputEvent.DataAs(data); err != nil {
		log.Printf("Got error while unmarshalling data: %s", err.Error())
		return cehttp.NewResult(400, "got error while unmarshalling data: %w", err)
	}

	// append entry time to event
	data.Times = append(data.Times, entryTime)

	// we're done, nowhere else to send
	if sink == "" {
		//b := make([]string, len(event.Times))
		//for i,t := range event.Times {
		//  b[i] = strconv.Itoa(t)
		//	event.Times[i] - event.Times[i-1]
		//}
		//log.Println("Event done", strings.Join(b, ","))
		log.Println("Event done", data.Times)
		return cehttp.NewResult(200, "")
	}

	// parse int from subject
	var functionNum int
	_, err := fmt.Sscanf(inputEvent.Subject(), "function%d", &functionNum)
	if err != nil {
		log.Println("Can't parse function number from subject")
		return cehttp.NewResult(404, "Can't parse function number from subject")
	}
	if functionNum > len(chEvent) {
		msg := fmt.Sprintf("Requested function number %d, but only got %d functions", functionNum, len(chEvent))
		log.Println(msg)
		return cehttp.NewResult(404, msg)
	}
	chEvent[functionNum-1] <- *data
	return cehttp.NewResult(200, "")
}

func Function(functionNum int, chJobs <-chan my.Event, chQuit <-chan bool, chDone chan<- bool) {
	/*
		client := &http.Client{
			Transport: &http.Transport{
				MaxIdleConnsPerHost: MaxIdleConnections,
			},
			Timeout: time.Duration(RequestTimeout) * time.Second,
		}
	*/
	var c cloudevents.Client
	if len(sink) > 0 {
		p, err := cloudevents.NewHTTP(cloudevents.WithTarget(sink))
		if err != nil {
			log.Fatalf("failed to create http protocol: %s", err.Error())
		}

		c, err = cloudevents.NewClient(p, cloudevents.WithUUIDs(), cloudevents.WithTimeNow())
		if err != nil {
			log.Fatalf("failed to create client: %s", err.Error())
		}
	}

	eventSource := fmt.Sprintf("https://github.com/knix-microfunctions/workflowmesh/event-performance/cmd/heartbeats/#function%d", functionNum)
	eventSubject := fmt.Sprintf("function%d", functionNum+1)
	defer close(chDone)
	for {
		select {
		case <-chQuit:
			return
		case data := <-chJobs:
			// Do some work on the event
			/*
				for i := 100000000; i > 0; i-- {
					i = i - rand.Intn(10)
				}
			*/
			if c == nil {
				log.Printf("Function%d done, channel backlog %d", functionNum, len(chJobs))
				continue
			}
			event := cloudevents.NewEvent("1.0")
			event.SetType(eventType)
			event.SetSource(eventSource)
			event.SetSubject(eventSubject)

			exitTime := time.Now().UnixNano()
			data.Times = append(data.Times, exitTime)
			if err := event.SetData(cloudevents.ApplicationJSON, data); err != nil {
				log.Printf("failed to set cloudevents data: %s", err.Error())
			}

			// log.Printf("sending cloudevent to %s", sink)
			if res := c.Send(context.Background(), event); !cloudevents.IsACK(res) {
				log.Printf("failed to send cloudevent: %v", res)
			}
			/*
				body, err := json.Marshal(data)
				req, err := http.NewRequest("POST", env.Sink+fmt.Sprintf("function%d", functionNum+1), bytes.NewBuffer(body))
				if err != nil {
					log.Fatalf("Error Occured. %+v", err)
				}
				req.Header.Set("Content-Type", "application/json")

				response, err := client.Do(req)
				if err != nil && response == nil {
					log.Printf("Error sending request to sink. %+v", err)
					continue
				}
				// Let's check if the work actually is done
				// We have seen inconsistencies even when we get 200 OK response
				_, err = ioutil.ReadAll(response.Body)
				if err != nil {
					log.Fatalf("Couldn't parse response body. %+v", err)
				}
				response.Body.Close()
			*/
		}
	}
}

func main() {
	flag.Parse()
	var env envConfig
	if err := envconfig.Process("", &env); err != nil {
		log.Printf("[ERROR] Failed to process env var: %s", err)
		os.Exit(1)
	}
	if env.Sink != "" {
		sink = env.Sink
	}
	if len(sink) > 0 && sink[len(sink)-1] != '/' {
		sink = sink + "/"
	}
	// Setup functions
	var chQuit [NumFunctions]chan bool
	var chDone [NumFunctions]chan bool
	for i := 0; i < NumFunctions; i++ {
		chEvent[i] = make(chan my.Event, 10)
		chQuit[i] = make(chan bool, 1)
		chDone[i] = make(chan bool, 1)
		go Function(i+1, chEvent[i], chQuit[i], chDone[i])
	}

	c, err := cloudevents.NewDefaultClient()
	if err != nil {
		log.Fatalf("failed to create client, %v", err)
	}
	log.Printf("listening on 8080 to respond to events")
	log.Fatalf("failed to start receiver: %s", c.StartReceiver(context.Background(), handleEvent))
	/*
		// HTTP Server
		mux := http.NewServeMux()
		mux.Handle("/data", http.FileServer(http.Dir(env.DataDir)))
		mux.HandleFunc("/", handleRequest)
		httpServer := &http.Server{
			Addr:    "0.0.0.0:8080",
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
			for i := 0; i < NumFunctions; i++ {
				chQuit[i] <- true
			}
			for i := 0; i < NumFunctions; i++ {
				<-chDone[i]
			}
			close(done)
			close(quit)
		}()

		log.Printf("listening on 8080, processing and sending events to %s", env.Sink)
		if err := httpServer.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			fmt.Printf("ListenAndServe Error %+v", err)
		}
		<-done */
}
