// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

import { useEffect, useMemo, useState } from "react";
import {
  ActionIcon,
  Badge,
  Box,
  Button,
  Card,
  Checkbox,
  Divider,
  Group,
  Loader,
  Paper,
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
  IconRefresh,
  IconStethoscope,
} from "@tabler/icons-react";

import { calculate, type CalculationResponse } from "../api/calc";

/**
 * Wells Score for Pulmonary Embolism: seven weighted clinical criteria,
 * guiding D-dimer vs CTPA (NICE NG158).
 */

type Inputs = {
  clinical_signs_of_dvt: boolean;
  pe_most_likely_diagnosis: boolean;
  heart_rate_over_100: boolean;
  immobilisation_or_surgery: boolean;
  previous_dvt_or_pe: boolean;
  haemoptysis: boolean;
  malignancy: boolean;
};

const blankInputs = (): Inputs => ({
  clinical_signs_of_dvt: false,
  pe_most_likely_diagnosis: false,
  heart_rate_over_100: false,
  immobilisation_or_surgery: false,
  previous_dvt_or_pe: false,
  haemoptysis: false,
  malignancy: false,
});

const CRITERIA: Array<{
  key: keyof Inputs;
  label: string;
  hint: string;
  points: string;
}> = [
  {
    key: "clinical_signs_of_dvt",
    label: "Clinical signs and symptoms of DVT",
    hint: "At minimum, leg swelling and pain with palpation of the deep veins.",
    points: "+3",
  },
  {
    key: "pe_most_likely_diagnosis",
    label: "PE is the most likely diagnosis",
    hint: "PE is at least as likely as any alternative diagnosis.",
    points: "+3",
  },
  {
    key: "heart_rate_over_100",
    label: "Heart rate > 100 bpm",
    hint: "",
    points: "+1.5",
  },
  {
    key: "immobilisation_or_surgery",
    label: "Immobilisation >= 3 days, or surgery in the previous 4 weeks",
    hint: "",
    points: "+1.5",
  },
  {
    key: "previous_dvt_or_pe",
    label: "Previous objectively diagnosed DVT or PE",
    hint: "",
    points: "+1.5",
  },
  {
    key: "haemoptysis",
    label: "Haemoptysis",
    hint: "",
    points: "+1",
  },
  {
    key: "malignancy",
    label: "Malignancy",
    hint: "Treatment within the last 6 months, or palliative.",
    points: "+1",
  },
];

function asString(v: unknown): string {
  if (v === null || v === undefined) return "";
  if (typeof v === "string") return v;
  return String(v);
}

function buildClipboardSummary(r: CalculationResponse, inputs: Inputs): string {
  const working = r.working ?? {};
  const twoTier = asString(working.two_tier);
  const threeTier = asString(working.three_tier);
  const selected = CRITERIA.filter((c) => inputs[c.key])
    .map((c) => `- ${c.label} (${c.points})`)
    .join("\n");

  return [
    `Wells Score (PE) ${asString(r.result)}`,
    "",
    selected ? `Positive criteria:\n${selected}` : "No criteria met.",
    "",
    r.interpretation ?? "",
    "",
    `Two-tier: ${twoTier || "—"}`,
    `Three-tier: ${threeTier || "—"}`,
    `Reference: ${r.reference ?? ""}`,
  ].join("\n");
}

function tierColour(twoTier: string): string {
  return twoTier === "likely" ? "red" : "teal";
}

export function WellsPeCalculator() {
  const [inputs, setInputs] = useState<Inputs>(blankInputs());
  const [response, setResponse] = useState<CalculationResponse | null>(null);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [clipboardText, setClipboardText] = useState("");

  useEffect(() => {
    let cancelled = false;
    setPending(true);
    setError(null);
    calculate("wells_pe", inputs as unknown as Record<string, unknown>)
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
  const twoTier = asString(working.two_tier);
  const threeTier = asString(working.three_tier);
  const colour = useMemo(() => tierColour(twoTier), [twoTier]);

  const reset = () => setInputs(blankInputs());

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(clipboardText);
      notifications.show({
        title: "Copied",
        message: "Wells (PE) result copied.",
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
    <Stack gap="xl" maw={960}>
      <Group justify="space-between" align="flex-start">
        <Box>
          <Group gap="sm" mb={4}>
            <ThemeIcon size="lg" variant="light" color="teal" radius="md">
              <IconStethoscope size={22} />
            </ThemeIcon>
            <Title order={2}>Wells Score (PE)</Title>
            <Badge color="teal" variant="light">
              Pulmonary embolism
            </Badge>
          </Group>
          <Text c="dimmed" size="sm">
            Pretest probability of pulmonary embolism, guiding D-dimer versus
            CTPA (NICE NG158).
          </Text>
        </Box>
        <Tooltip label="Reset all criteria">
          <ActionIcon variant="subtle" color="gray" onClick={reset} size="lg">
            <IconRefresh size={18} />
          </ActionIcon>
        </Tooltip>
      </Group>

      <Group align="stretch" gap="lg" wrap="nowrap">
        <Card withBorder padding="lg" radius="lg" style={{ flex: 1 }}>
          <Stack gap="sm">
            <Text fw={600}>Tick each criterion that applies</Text>
            <Divider />
            {CRITERIA.map((c) => (
              <Checkbox
                key={c.key}
                size="md"
                label={
                  <Group gap="xs">
                    <Text>{c.label}</Text>
                    <Badge size="xs" variant="light" color="gray">
                      {c.points}
                    </Badge>
                  </Group>
                }
                description={c.hint || undefined}
                checked={inputs[c.key]}
                onChange={(e) => {
                  const checked = e.currentTarget.checked;
                  setInputs((prev) => ({ ...prev, [c.key]: checked }));
                }}
              />
            ))}
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
                  <Badge ml="auto" color={colour} variant="light" size="lg">
                    {twoTier || "—"}
                  </Badge>
                </Group>
                <Text size="sm" style={{ lineHeight: 1.55 }}>
                  {response.interpretation}
                </Text>
                <Divider my={4} />
                <Group gap="xs" wrap="wrap">
                  <Badge variant="outline" color="gray">
                    Three-tier: {threeTier || "—"}
                  </Badge>
                </Group>
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
              minRows={9}
              maxRows={9}
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
