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
  NumberInput,
  Paper,
  SegmentedControl,
  SimpleGrid,
  Stack,
  Text,
  Textarea,
  ThemeIcon,
  Title,
  Tooltip,
} from "@mantine/core";
import { notifications } from "@mantine/notifications";
import {
  IconActivity,
  IconAlertCircle,
  IconCheck,
  IconCopy,
  IconRefresh,
} from "@tabler/icons-react";

import { calculate, type CalculationResponse } from "../api/calc";

/**
 * NEWS2: NHS-mandated aggregate physiology score (RCP 2017) driving the
 * clinical-response band. Scale 2 (hypercapnic respiratory failure, e.g.
 * COPD, with a prescribed 88-92% target) is a distinct toggle because it
 * changes how the same SpO2 reading is scored.
 */

type Consciousness = "alert" | "confusion-or-vpu";

type Inputs = {
  respiratory_rate: number;
  spo2: number;
  spo2_scale: "1" | "2";
  on_oxygen: boolean;
  systolic_bp: number;
  pulse: number;
  consciousness: Consciousness;
  temperature: number;
};

const blankInputs = (): Inputs => ({
  respiratory_rate: 16,
  spo2: 98,
  spo2_scale: "1",
  on_oxygen: false,
  systolic_bp: 120,
  pulse: 75,
  consciousness: "alert",
  temperature: 36.8,
});

function asString(v: unknown): string {
  if (v === null || v === undefined) return "";
  if (typeof v === "string") return v;
  return String(v);
}

function bandColour(band: string): string {
  if (band === "high") return "red";
  if (band === "medium") return "orange";
  if (band === "low-medium") return "yellow";
  if (band === "low") return "yellow";
  return "teal";
}

function buildClipboardSummary(r: CalculationResponse, inputs: Inputs): string {
  const working = r.working ?? {};
  const band = asString(working.band);

  return [
    `NEWS2 ${asString(r.result)} (${band})`,
    "",
    `Respiratory rate: ${inputs.respiratory_rate}/min (${asString(working.respiratory_rate_score ?? "")})`,
    `SpO2: ${inputs.spo2}% on scale ${inputs.spo2_scale}${inputs.on_oxygen ? ", on oxygen" : ", on air"} (${asString(working.spo2_score ?? "")})`,
    `Air/oxygen: ${asString(working.air_or_oxygen_score ?? "")}`,
    `Systolic BP: ${inputs.systolic_bp} mmHg (${asString(working.systolic_bp_score ?? "")})`,
    `Pulse: ${inputs.pulse} bpm (${asString(working.pulse_score ?? "")})`,
    `Consciousness: ${inputs.consciousness} (${asString(working.consciousness_score ?? "")})`,
    `Temperature: ${inputs.temperature} degC (${asString(working.temperature_score ?? "")})`,
    "",
    r.interpretation ?? "",
  ].join("\n");
}

export function News2Calculator() {
  const [inputs, setInputs] = useState<Inputs>(blankInputs());
  const [response, setResponse] = useState<CalculationResponse | null>(null);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [clipboardText, setClipboardText] = useState("");

  useEffect(() => {
    let cancelled = false;
    setPending(true);
    setError(null);
    calculate("news2", inputs as unknown as Record<string, unknown>)
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
  const band = asString(working.band);
  const colour = useMemo(() => bandColour(band), [band]);
  const singleParameter3 = working.single_parameter_3 === true;

  const reset = () => setInputs(blankInputs());

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(clipboardText);
      notifications.show({
        title: "Copied",
        message: "NEWS2 result copied.",
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
              <IconActivity size={22} />
            </ThemeIcon>
            <Title order={2}>NEWS2</Title>
            <Badge color="teal" variant="light">
              NHS-mandated
            </Badge>
          </Group>
          <Text c="dimmed" size="sm">
            National Early Warning Score 2 (RCP 2017): aggregate physiology
            score driving the clinical-response band.
          </Text>
        </Box>
        <Tooltip label="Reset to normal observations">
          <ActionIcon variant="subtle" color="gray" onClick={reset} size="lg">
            <IconRefresh size={18} />
          </ActionIcon>
        </Tooltip>
      </Group>

      <Group align="stretch" gap="lg" wrap="nowrap">
        <Card withBorder padding="lg" radius="lg" style={{ flex: 1 }}>
          <Stack gap="md">
            <SimpleGrid cols={2} spacing="md">
              <NumberInput
                label="Respiratory rate"
                description="breaths/min"
                min={0}
                max={60}
                value={inputs.respiratory_rate}
                onChange={(value) =>
                  setInputs((prev) => ({
                    ...prev,
                    respiratory_rate: Number(value) || 0,
                  }))
                }
              />
              <NumberInput
                label="SpO2"
                description="%"
                min={0}
                max={100}
                value={inputs.spo2}
                onChange={(value) =>
                  setInputs((prev) => ({ ...prev, spo2: Number(value) || 0 }))
                }
              />
            </SimpleGrid>

            <Box>
              <Text fw={500} size="sm" mb={4}>
                SpO2 scale
              </Text>
              <SegmentedControl
                fullWidth
                value={inputs.spo2_scale}
                onChange={(value) =>
                  setInputs((prev) => ({
                    ...prev,
                    spo2_scale: value as "1" | "2",
                  }))
                }
                data={[
                  { value: "1", label: "Scale 1 (default)" },
                  { value: "2", label: "Scale 2 (COPD, target 88-92%)" },
                ]}
              />
            </Box>

            <Checkbox
              size="md"
              label="Receiving supplemental oxygen"
              checked={inputs.on_oxygen}
              onChange={(e) => {
                const checked = e.currentTarget.checked;
                setInputs((prev) => ({ ...prev, on_oxygen: checked }));
              }}
            />

            <Divider />

            <SimpleGrid cols={2} spacing="md">
              <NumberInput
                label="Systolic BP"
                description="mmHg"
                min={0}
                max={300}
                value={inputs.systolic_bp}
                onChange={(value) =>
                  setInputs((prev) => ({
                    ...prev,
                    systolic_bp: Number(value) || 0,
                  }))
                }
              />
              <NumberInput
                label="Pulse"
                description="bpm"
                min={0}
                max={250}
                value={inputs.pulse}
                onChange={(value) =>
                  setInputs((prev) => ({ ...prev, pulse: Number(value) || 0 }))
                }
              />
            </SimpleGrid>

            <NumberInput
              label="Temperature"
              description="degC"
              min={25}
              max={43}
              step={0.1}
              decimalScale={1}
              value={inputs.temperature}
              onChange={(value) =>
                setInputs((prev) => ({
                  ...prev,
                  temperature: Number(value) || 0,
                }))
              }
            />

            <Box>
              <Text fw={500} size="sm" mb={4}>
                Consciousness (ACVPU)
              </Text>
              <SegmentedControl
                fullWidth
                value={inputs.consciousness}
                onChange={(value) =>
                  setInputs((prev) => ({
                    ...prev,
                    consciousness: value as Consciousness,
                  }))
                }
                data={[
                  { value: "alert", label: "Alert" },
                  { value: "confusion-or-vpu", label: "New confusion / V / P / U" },
                ]}
              />
            </Box>
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
                    {band || "—"}
                  </Badge>
                </Group>
                {singleParameter3 && (
                  <Badge color="yellow" variant="light" w="fit-content">
                    Single red score (3)
                  </Badge>
                )}
                <Text size="sm" style={{ lineHeight: 1.55 }}>
                  {response.interpretation}
                </Text>
                <Divider my={4} />
                <Stack gap={4}>
                  <Text size="xs" c="dimmed" tt="uppercase" fw={600}>
                    Per-parameter scores
                  </Text>
                  <Group gap="xs" wrap="wrap">
                    <Badge variant="outline" color="gray">
                      RR: {asString(working.respiratory_rate_score)}
                    </Badge>
                    <Badge variant="outline" color="gray">
                      SpO2: {asString(working.spo2_score)}
                    </Badge>
                    <Badge variant="outline" color="gray">
                      O2: {asString(working.air_or_oxygen_score)}
                    </Badge>
                    <Badge variant="outline" color="gray">
                      SBP: {asString(working.systolic_bp_score)}
                    </Badge>
                    <Badge variant="outline" color="gray">
                      Pulse: {asString(working.pulse_score)}
                    </Badge>
                    <Badge variant="outline" color="gray">
                      Conscious: {asString(working.consciousness_score)}
                    </Badge>
                    <Badge variant="outline" color="gray">
                      Temp: {asString(working.temperature_score)}
                    </Badge>
                  </Group>
                </Stack>
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
              minRows={11}
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
