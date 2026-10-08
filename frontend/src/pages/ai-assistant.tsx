import { useEffect, useRef, useState } from "react";
import { Send, Sparkles } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardContent } from "@/components/ui/card";
import { cn } from "@/lib/utils";
import { useSendAiMessage } from "@/hooks/use-ai";
import { type AiChatResponse } from "@/lib/api";

interface ChatMessage {
  role: "user" | "assistant";
  content: string;
  tool_calls?: AiChatResponse["tool_calls"];
}

export function AiAssistantPage() {
  const [sessionId, setSessionId] = useState<string | undefined>(undefined);
  const [input, setInput] = useState("");
  const [messages, setMessages] = useState<ChatMessage[]>([
    {
      role: "assistant",
      content:
        "Hi! I can answer questions about your contacts, deals, invoices, and employees. What would you like to know?",
    },
  ]);

  const sendMessage = useSendAiMessage();
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, sendMessage.isPending]);

  const handleSubmit = (event: React.FormEvent) => {
    event.preventDefault();
    if (!input.trim() || sendMessage.isPending) return;

    const userMessage = input.trim();
    setInput("");
    setMessages((prev) => [...prev, { role: "user", content: userMessage }]);

    sendMessage.mutate(
      { session_id: sessionId, message: userMessage },
      {
        onSuccess: (response) => {
          setSessionId(response.session_id);
          setMessages((prev) => [
            ...prev,
            {
              role: "assistant",
              content: response.message,
              tool_calls: response.tool_calls,
            },
          ]);
        },
        onError: (error) => {
          setMessages((prev) => [
            ...prev,
            {
              role: "assistant",
              content: `Sorry, something went wrong: ${error.message}`,
            },
          ]);
        },
      }
    );
  };

  return (
    <div className="flex h-[calc(100vh-8rem)] flex-col gap-4">
      <div>
        <h2 className="text-2xl font-semibold tracking-tight">AI Assistant</h2>
        <p className="text-muted-foreground">
          Ask about your business data. I can only read, never modify.
        </p>
      </div>

      <Card className="flex flex-1 flex-col overflow-hidden">
        <CardContent className="flex flex-1 flex-col gap-4 overflow-y-auto p-4">
          {messages.map((message, index) => (
            <div
              key={index}
              className={cn(
                "flex w-full",
                message.role === "user" ? "justify-end" : "justify-start"
              )}
            >
              <div
                className={cn(
                  "max-w-[80%] rounded-lg px-4 py-2 text-sm",
                  message.role === "user"
                    ? "bg-primary text-primary-foreground"
                    : "bg-muted"
                )}
              >
                {message.role === "assistant" && (
                  <Sparkles className="mb-1 inline h-3.5 w-3.5 opacity-70" />
                )}{" "}
                {message.content}
                {message.tool_calls && message.tool_calls.length > 0 && (
                  <div className="mt-2 space-y-1 border-t border-current/20 pt-2 text-xs opacity-80">
                    <p className="font-medium">Tools used:</p>
                    {message.tool_calls.map((tool, idx) => (
                      <p key={idx}>
                        {tool.tool_name}
                        {tool.error_message && (
                          <span className="text-destructive">
                            {" "}
                            ({tool.error_message})
                          </span>
                        )}
                      </p>
                    ))}
                  </div>
                )}
              </div>
            </div>
          ))}
          {sendMessage.isPending && (
            <div className="flex justify-start">
              <div className="max-w-[80%] rounded-lg bg-muted px-4 py-2 text-sm">
                <Sparkles className="inline h-3.5 w-3.5 animate-pulse opacity-70" />{" "}
                Thinking...
              </div>
            </div>
          )}
          <div ref={bottomRef} />
        </CardContent>

        <div className="border-t p-4">
          <form onSubmit={handleSubmit} className="flex gap-2">
            <Input
              placeholder="e.g. Who are our customers?"
              value={input}
              onChange={(e) => setInput(e.target.value)}
              disabled={sendMessage.isPending}
              className="flex-1"
            />
            <Button type="submit" disabled={sendMessage.isPending || !input.trim()}>
              <Send className="h-4 w-4" />
              <span className="sr-only">Send</span>
            </Button>
          </form>
        </div>
      </Card>
    </div>
  );
}
