// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

import { useEffect, useMemo, useState } from "react";
import {
  ActionIcon,
  Alert,
  Badge,
  Box,
  Button,
  Card,
  Divider,
  Group,
  Loader,
  Paper,
  SegmentedControl,
  Stack,
  Text,
  Textarea,
  ThemeIcon,
  Title,
  Tooltip,
} from "@mantine/core";
import { notifications } from "@mantine/notifications";
import {
  IconAlertCircle,
  IconCheck,
  IconCopy,
  IconMoodSad,
  IconRefresh,
} from "@tabler/icons-react";

import { calculate, type CalculationResponse } from "../api/calc";

/**
 * PHQ-9: nine-item depression severity measure over the last two weeks
 * (Kroenke, Spitzer & Williams 2001; public domain, Pfizer 2010). Item 9
 * is a clinical-safety item (thoughts of self-harm) and is visually
 * separated and flagged independently of the total score.
 */

const ITEMS = [
  "Little interest or pleasure in doing things",
  "Feeling down, depressed, or hopeless",
  "Trouble falling or staying asleep, or sleeping too much",
  "Feeling tired or having little energy",
  "Poor appetite or overeating",
  "Feeling bad about yourself - or that you are a failure, or have let yourself or your family down",
  "Trouble concentrating on things, such as reading or watching television",
  "Moving or speaking so slowly that other people could have noticed, or being so fidgety or restless that you have been moving a lot more than usual",
  "Thoughts that you would be better off dead, or of hurting yourself in some way",
];

const RESPONSE_OPTIONS = [
  { value: "0", label: "Not at all" },
  { value: "1", label: "Several days" },
  { value: "2", label: "More than half the days" },
  { value: "3", label: "Nearly every day" },
];

const SELF_HARM_ITEM = 8;

type Inputs = { responses: number[] };

const blankInputs = (): Inputs => ({ responses: Array(9).fill(0) });

function asString(v: unknown): string {
  if (v === null || v === undefined) return "";
  if (typeof v === "string") return v;
  return String(v);
}

function severityLabel(slug: string): string {
  const labels: Record<string, string> = {
    "none-minimal": "none-minimal",
    mild: "mild",
    moderate: "moderate",
    "moderately-severe": "moderately severe",
    severe: "severe",
  };
  return labels[slug] ?? slug;
}

function severityColour(slug: string): string {
  if (slug === "severe" || slug === "moderately-severe") return "red";
  if (slug === "moderate") return "yellow";
  if (slug === "mild") return "yellow";
  return "teal";
}

function buildClipboardSummary(r: CalculationResponse, inputs: Inputs): string {
  const working = r.working ?? {};
  const severity = severityLabel(asString(working.severity));
  const selfHarm = working.self_harm_item_flag === true;

  const itemLines = ITEMS.map(
    (item, i) =>
      `Q${i + 1} (${inputs.responses[i]}/3): ${item.slice(0, 60)}${item.length > 60 ? "..." : ""}`,
  ).join("\n");

  return [
    `PHQ-9 ${asString(r.result)} / 27 (${severity})`,
    "",
    r.interpretation ?? "",
    "",
    selfHarm
      ? "** Item 9 positive: suicide-risk assessment indicated **"
      : "Item 9 (self-harm): negative.",
    "",
    itemLines,
    "",
    `Reference: ${r.reference ?? ""}`,
  ].join("\n");
}

export function Phq9Calculator() {
  const [inputs, setInputs] = useState<Inputs>(blankInputs());
  const [response, setResponse] = useState<CalculationResponse | null>(null);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [clipboardText, setClipboardText] = useState("");

  useEffect(() => {
    let cancelled = false;
    setPending(true);
    setError(null);
    calculate("phq9", inputs as unknown as Record<string, unknown>)
      .then((r) => {
        if (cancelled) return;
        setResponse(r);
        setClipboardText(buildClipboardSummary(r, inputs));
      })
      .catch((e: unknown) => {
        if (cancelled) return;
        setError(String(e));
        setResponse(null);
      })
      .finally(() => !cancelled && setPending(false));
    return () => {
      cancelled = true;
    };
  }, [inputs]);

  const working = response?.working ?? {};
  const severitySlug = asString(working.severity);
  const selfHarm = working.self_harm_item_flag === true;
  const colour = useMemo(() => severityColour(severitySlug), [severitySlug]);

  const setItem = (index: number, value: string) => {
    setInputs((prev) => {
      const responses = [...prev.responses];
      responses[index] = Number(value);
      return { responses };
    });
  };

  const reset = () => setInputs(blankInputs());

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(clipboardText);
      notifications.show({
        title: "Copied",
        message: "PHQ-9 result copied.",
        color: "teal",
        icon: <IconCheck size={18} />,
        autoClose: 2000,
      });
    } catch {
      notifications.show({
        title: "Copy failed",
        message: "Select the text above and copy manually.",
        color: "red",
        icon: <IconAlertCircle size={18} />,
      });
    }
  };

  return (
    <Stack gap="xl" maw={980}>
      <Group justify="space-between" align="flex-start">
        <Box>
          <Group gap="sm" mb={4}>
            <ThemeIcon size="lg" variant="light" color="teal" radius="md">
              <IconMoodSad size={22} />
            </ThemeIcon>
            <Title order={2}>PHQ-9</Title>
            <Badge color="teal" variant="light">
              Depression severity
            </Badge>
          </Group>
          <Text c="dimmed" size="sm">
            Nine-item depression severity score over the last two weeks
            (Kroenke, Spitzer &amp; Williams 2001). Public domain - released
            by Pfizer.
          </Text>
        </Box>
        <Tooltip label="Reset all items">
          <ActionIcon variant="subtle" color="gray" onClick={reset} size="lg">
            <IconRefresh size={18} />
          </ActionIcon>
        </Tooltip>
      </Group>

      <Group align="stretch" gap="lg" wrap="nowrap">
        <Card withBorder padding="lg" radius="lg" style={{ flex: 1.3 }}>
          <Stack gap="md">
            <Text fw={600}>
              Over the last 2 weeks, how often have you been bothered by:
            </Text>
            <Divider />
            <Stack gap="md">
              {ITEMS.map((item, i) => (
                <Box key={i}>
                  <Text
                    size="sm"
                    fw={i === SELF_HARM_ITEM ? 600 : 400}
                    c={i === SELF_HARM_ITEM ? "red" : undefined}
                    mb={6}
                  >
                    Q{i + 1}. {item}
                  </Text>
                  <SegmentedControl
                    fullWidth
                    size="xs"
                    value={String(inputs.responses[i])}
                    onChange={(value) => setItem(i, value)}
                    data={RESPONSE_OPTIONS}
                    color={i === SELF_HARM_ITEM ? "red" : "teal"}
                  />
                </Box>
              ))}
            </Stack>
          </Stack>
        </Card>

        <Card
          withBorder
          padding="lg"
          radius="lg"
          style={{ flex: 1 }}
          bg="var(--mantine-color-default-hover)"
        >
          <Stack gap="md" h="100%">
            <Text fw={600} c="dimmed" size="sm" tt="uppercase">
              Result
            </Text>
            {pending && !response && (
              <Group gap="xs">
                <Loader size="xs" />
                <Text c="dimmed">Computing…</Text>
              </Group>
            )}
            {error && (
              <Text c="red" size="sm">
                {error}
              </Text>
            )}
            {response && (
              <>
                <Group align="baseline" gap="xs">
                  <Title order={1} c={colour} fz={64} lh={1}>
                    {String(response.result)}
                  </Title>
                  <Text c="dimmed" fz="lg">
                    / 27
                  </Text>
                </Group>
                <Badge color={colour} variant="light" size="lg" w="fit-content">
                  {severityLabel(severitySlug) || "—"}
                </Badge>
                <Text size="sm" style={{ lineHeight: 1.55 }}>
                  {response.interpretation}
                </Text>
                {selfHarm && (
                  <Alert
                    color="red"
                    icon={<IconAlertCircle size={18} />}
                    title="Item 9 positive"
                  >
                    Thoughts of self-harm reported - a risk assessment is
                    indicated regardless of the total score.
                  </Alert>
                )}
              </>
            )}
          </Stack>
        </Card>
      </Group>

      {response && (
        <Paper
          withBorder
          radius="lg"
          p="lg"
          className="clipboard-preview"
          style={{ borderColor: "var(--mantine-color-teal-4)", borderWidth: 2 }}
        >
          <Stack gap="sm">
            <Group justify="space-between" align="center">
              <Box>
                <Text fw={700} size="md">
                  Paste-ready summary
                </Text>
                <Text size="xs" c="dimmed">
                  Edit freely before copying - your edits are preserved.
                </Text>
              </Box>
              <Button
                leftSection={<IconCopy size={16} />}
                color="teal"
                onClick={copy}
              >
                Copy result
              </Button>
            </Group>
            <Textarea
              autosize
              minRows={10}
              maxRows={14}
              value={clipboardText}
              onChange={(e) => setClipboardText(e.currentTarget.value)}
              styles={{ input: { background: "var(--mantine-color-body)" } }}
            />
            <Text size="xs" c="dimmed">
              Reference: {response.reference}
            </Text>
          </Stack>
        </Paper>
      )}
    </Stack>
  );
}
