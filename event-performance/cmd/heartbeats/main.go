/*
Copyright 2018 The Knative Authors

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

package main

import (
	"context"
	"crypto/rand"
	"flag"
	"fmt"
	"log"
	"os"
	"strconv"
	"time"

	cloudevents "github.com/cloudevents/sdk-go/v2"
	"github.com/kelseyhightower/envconfig"
	"github.com/knix-microfunctions/workflowmesh/event-performance/my"
)

const (
	MaxIdleConnections int = 20
	RequestTimeout     int = 5
)

var (
	eventSource string
	eventType   string
	sink        string
	subject     string
	label       string
	periodStr   string
)

func init() {
	flag.StringVar(&eventSource, "eventSource", "", "the event-source (CloudEvents)")
	flag.StringVar(&eventType, "eventType", "dev.knative.eventing.samples.heartbeat", "the event-type (CloudEvents)")
	flag.StringVar(&sink, "sink", "", "the host url to heartbeat to")
	flag.StringVar(&subject, "subject", "", "the subject of the events")
	flag.StringVar(&label, "label", "", "a special label")
	flag.StringVar(&periodStr, "period", "5", "the number of seconds between heartbeats")
}

type envConfig struct {
	// Sink URL where to send heartbeat cloudevents
	Sink string `envconfig:"K_SINK"`

	// CEOverrides are the CloudEvents overrides to be applied to the outbound event.
	CEOverrides string `envconfig:"K_CE_OVERRIDES"`

	// Name of this pod.
	Name string `envconfig:"POD_NAME" required:"true"`

	// Namespace this pod exists in.
	Namespace string `envconfig:"POD_NAMESPACE" required:"true"`

	// Whether to run continuously or exit.
	OneShot bool `envconfig:"ONE_SHOT" default:"false"`

	// Number of random bytes in data
	DataSize int `envconfig:"DATASIZE"`

	// Function name
	Subject string `envconfig:"SUBJECT"`
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
	if env.Subject != "" {
		subject = env.Subject
	}

	p, err := cloudevents.NewHTTP(cloudevents.WithTarget(sink))
	if err != nil {
		log.Fatalf("failed to create http protocol: %s", err.Error())
	}

	c, err := cloudevents.NewClient(p, cloudevents.WithUUIDs(), cloudevents.WithTimeNow())
	if err != nil {
		log.Fatalf("failed to create client: %s", err.Error())
	}

	/*
		client := &http.Client{
			Transport: &http.Transport{
				MaxIdleConnsPerHost: MaxIdleConnections,
			},
			Timeout: time.Duration(RequestTimeout) * time.Second,
		}
	*/

	var period time.Duration
	if p, err := strconv.Atoi(periodStr); err != nil {
		period = time.Duration(5) * time.Second
	} else {
		period = time.Duration(p) * time.Millisecond
	}

	if eventSource == "" {
		eventSource = fmt.Sprintf("https://github.com/knix-microfunctions/workflowmesh/event-performance/cmd/heartbeats/#%s/%s", env.Namespace, env.Name)
		log.Printf("Event Source: %s", eventSource)
	}

	if len(label) > 0 && label[0] == '"' {
		label, _ = strconv.Unquote(label)
	}
	data := make([]byte, env.DataSize)
	rand.Read(data)
	hb := &my.Event{
		Times: []int64{0},
		Data:  data,
	}
	ticker := time.NewTicker(period)

	for {
		hb.Times[0] = time.Now().UnixNano()
		/*
			body, err := json.Marshal(hb)
			if err != nil {
				log.Fatalf("Error Occured. %+v", err)
			}
		*/

		event := cloudevents.NewEvent("1.0")
		event.SetType(eventType)
		event.SetSource(eventSource)
		event.SetSubject(subject)

		if err := event.SetData(cloudevents.ApplicationJSON, hb); err != nil {
			log.Printf("failed to set cloudevents data: %s", err.Error())
		}

		// log.Printf("sending cloudevent to %s", sink)
		if res := c.Send(context.Background(), event); !cloudevents.IsACK(res) {
			log.Printf("failed to send cloudevent: %v", res)
		}

		/*
			req, err := http.NewRequest("POST", sink, bytes.NewBuffer(body))
			if err != nil {
				log.Fatalf("Error Occured. %+v", err)
			}
			req.Header.Set("Content-Type", "application/json")

			log.Println("Sending request to " + sink)
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

		if env.OneShot {
			return
		}

		// Wait for next tick
		<-ticker.C
	}
}
