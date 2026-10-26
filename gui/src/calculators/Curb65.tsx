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
  IconAlertCircle,
  IconCheck,
  IconCopy,
  IconLungs,
  IconRefresh,
} from "@tabler/icons-react";

import { calculate, type CalculationResponse } from "../api/calc";

/**
 * CURB-65: five-item severity and 30-day mortality score for
 * community-acquired pneumonia (Lim et al 2003; BTS 2009 / NICE NG250).
 */

type Inputs = {
  confusion: boolean;
  urea_mmol_l: number;
  respiratory_rate: number;
  systolic_bp: number;
  diastolic_bp: number;
  age: number;
};

const blankInputs = (): Inputs => ({
  confusion: false,
  urea_mmol_l: 5,
  respiratory_rate: 18,
  systolic_bp: 120,
  diastolic_bp: 75,
  age: 65,
});

function asString(v: unknown): string {
  if (v === null || v === undefined) return "";
  if (typeof v === "string") return v;
  return String(v);
}

function buildClipboardSummary(r: CalculationResponse, inputs: Inputs): string {
  const working = r.working ?? {};
  const band = asString(working.risk_band);
  const mortality = asString(working.mortality_30_day_percent);

  const criteria = [
    inputs.confusion && "Confusion",
    working.urea_gt_7_mmol_l === true && "Urea > 7 mmol/L",
    inputs.respiratory_rate >= 30 && "Respiratory rate >= 30/min",
    (inputs.systolic_bp < 90 || inputs.diastolic_bp <= 60) &&
      "Blood pressure (SBP<90 or DBP<=60)",
    working.age_ge_65 === true && "Age >= 65",
  ].filter(Boolean) as string[];

  return [
    `CURB-65 ${asString(r.result)} / 5`,
    "",
    criteria.length > 0
      ? `Positive criteria:\n${criteria.map((c) => `- ${c}`).join("\n")}`
      : "No criteria met.",
    "",
    r.interpretation ?? "",
    "",
    `Risk band: ${band || "—"}`,
    mortality ? `30-day mortality (Lim 2003 cohort): ${mortality}%` : "",
  ]
    .filter((line) => line !== "")
    .join("\n");
}

function bandColour(band: string): string {
  if (band === "high") return "red";
  if (band === "intermediate") return "yellow";
  return "teal";
}

export function Curb65Calculator() {
  const [inputs, setInputs] = useState<Inputs>(blankInputs());
  const [response, setResponse] = useState<CalculationResponse | null>(null);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [clipboardText, setClipboardText] = useState("");

  useEffect(() => {
    let cancelled = false;
    setPending(true);
    setError(null);
    calculate("curb65", inputs as unknown as Record<string, unknown>)
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
  const band = asString(working.risk_band);
  const mortality = asString(working.mortality_30_day_percent);
  const colour = useMemo(() => bandColour(band), [band]);

  const reset = () => setInputs(blankInputs());

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(clipboardText);
      notifications.show({
        title: "Copied",
        message: "CURB-65 result copied.",
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
              <IconLungs size={22} />
            </ThemeIcon>
            <Title order={2}>CURB-65</Title>
            <Badge color="teal" variant="light">
              Pneumonia severity
            </Badge>
          </Group>
          <Text c="dimmed" size="sm">
            Severity and 30-day mortality risk in community-acquired
            pneumonia, guiding place of care (BTS 2009 / NICE NG250).
          </Text>
        </Box>
        <Tooltip label="Reset to defaults">
          <ActionIcon variant="subtle" color="gray" onClick={reset} size="lg">
            <IconRefresh size={18} />
          </ActionIcon>
        </Tooltip>
      </Group>

      <Group align="stretch" gap="lg" wrap="nowrap">
        <Card withBorder padding="lg" radius="lg" style={{ flex: 1 }}>
          <Stack gap="md">
            <Checkbox
              size="md"
              label="New-onset confusion"
              description="New disorientation in person, place, or time - not chronic baseline impairment."
              checked={inputs.confusion}
              onChange={(e) => {
                const checked = e.currentTarget.checked;
                setInputs((prev) => ({ ...prev, confusion: checked }));
              }}
            />
            <Divider />
            <SimpleGrid cols={2} spacing="md">
              <NumberInput
                label="Urea"
                description="mmol/L; scores a point when > 7"
                min={0}
                step={0.1}
                decimalScale={1}
                value={inputs.urea_mmol_l}
                onChange={(value) =>
                  setInputs((prev) => ({
                    ...prev,
                    urea_mmol_l: Number(value) || 0,
                  }))
                }
              />
              <NumberInput
                label="Respiratory rate"
                description="breaths/min; scores when >= 30"
                min={0}
                max={80}
                value={inputs.respiratory_rate}
                onChange={(value) =>
                  setInputs((prev) => ({
                    ...prev,
                    respiratory_rate: Number(value) || 0,
                  }))
                }
              />
              <NumberInput
                label="Systolic BP"
                description="mmHg; scores (with diastolic) when < 90"
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
                label="Diastolic BP"
                description="mmHg; scores (with systolic) when <= 60"
                min={0}
                max={200}
                value={inputs.diastolic_bp}
                onChange={(value) =>
                  setInputs((prev) => ({
                    ...prev,
                    diastolic_bp: Number(value) || 0,
                  }))
                }
              />
              <NumberInput
                label="Age"
                description="years; scores a point when >= 65"
                min={18}
                max={120}
                value={inputs.age}
                onChange={(value) =>
                  setInputs((prev) => ({ ...prev, age: Number(value) || 0 }))
                }
              />
            </SimpleGrid>
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
                    / 5
                  </Text>
                  <Badge ml="auto" color={colour} variant="light" size="lg">
                    {band || "—"}
                  </Badge>
                </Group>
                <Text size="sm" style={{ lineHeight: 1.55 }}>
                  {response.interpretation}
                </Text>
                <Divider my={4} />
                <Group gap="xs" wrap="wrap">
                  <Badge variant="outline" color="gray">
                    30-day mortality: {mortality || "—"}%
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
