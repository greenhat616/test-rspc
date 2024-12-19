import { QueryClient } from "@tanstack/react-query";
import { WebsocketTransport, createClient } from "@rspc/client";
import { createReactQueryHooks } from "@rspc/react-query";

import type { Procedures } from "../bindings"; // These were the bindings exported from your Rust code!
import Inner from "./Inner";

// You must provide the generated types as a generic and create a transport (in this example we are using HTTP Fetch) so that the client knows how to communicate with your API.
const client = createClient<Procedures>({
  // Refer to the integration your using for the correct transport.
  transport: new WebsocketTransport("ws://localhost:4000/rspc/ws"),
});

const queryClient = new QueryClient();
export const rspc = createReactQueryHooks<Procedures>();

function App() {
  return (
    <rspc.Provider client={client} queryClient={queryClient}>
      <Inner />
    </rspc.Provider>
  );
}

export default App;
