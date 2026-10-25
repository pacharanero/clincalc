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
  Checkbox,
  Divider,
  Group,
  Loader,
  NumberInput,
  Paper,
  Select,
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
  IconAlertCircle,
  IconCheck,
  IconCopy,
  IconDroplet,
  IconRefresh,
} from "@tabler/icons-react";

import { calculate, type CalculationResponse } from "../api/calc";

/**
 * QDiabetes-2018: 10-year risk of developing type 2 diabetes. Three nested
 * models are selected by which optional reading, if any, is supplied -
 * never both. `polycystic_ovary_syndrome` and `gestational_diabetes` apply
 * to the female model only and are hidden (not merely disabled) for males,
 * matching how the Rust side documents them as ignored.
 */

type Sex = "male" | "female";
type Ethnicity =
  | "white_or_not_stated"
  | "indian"
  | "pakistani"
  | "bangladeshi"
  | "other_asian"
  | "black_caribbean"
  | "black_african"
  | "chinese"
  | "other_ethnic_group";
type Smoking =
  | "non_smoker"
  | "ex_smoker"
  | "light_smoker"
  | "moderate_smoker"
  | "heavy_smoker";
type OptionalTest = "none" | "fasting_glucose" | "hba1c";

type Inputs = {
  age: number;
  sex: Sex;
  ethnicity: Ethnicity;
  smoking: Smoking;
  height_cm: number;
  weight_kg: number;
  townsend: number;
  family_history_diabetes: boolean;
  cardiovascular_disease: boolean;
  treated_hypertension: boolean;
  learning_disability: boolean;
  bipolar_or_schizophrenia: boolean;
  regular_steroids: boolean;
  statin: boolean;
  atypical_antipsychotic: boolean;
  polycystic_ovary_syndrome: boolean;
  gestational_diabetes: boolean;
  optionalTest: OptionalTest;
  fasting_glucose_mmol_l: number;
  hba1c_mmol_mol: number;
};

const blankInputs = (): Inputs => ({
  age: 55,
  sex: "female",
  ethnicity: "white_or_not_stated",
  smoking: "non_smoker",
  height_cm: 165,
  weight_kg: 70,
  townsend: 0,
  family_history_diabetes: false,
  cardiovascular_disease: false,
  treated_hypertension: false,
  learning_disability: false,
  bipolar_or_schizophrenia: false,
  regular_steroids: false,
  statin: false,
  atypical_antipsychotic: false,
  polycystic_ovary_syndrome: false,
  gestational_diabetes: false,
  optionalTest: "none",
  fasting_glucose_mmol_l: 5,
  hba1c_mmol_mol: 35,
});

const ETHNICITY_OPTIONS: Array<{ value: Ethnicity; label: string }> = [
  { value: "white_or_not_stated", label: "White or not stated" },
  { value: "indian", label: "Indian" },
  { value: "pakistani", label: "Pakistani" },
  { value: "bangladeshi", label: "Bangladeshi" },
  { value: "other_asian", label: "Other Asian" },
  { value: "black_caribbean", label: "Black Caribbean" },
  { value: "black_african", label: "Black African" },
  { value: "chinese", label: "Chinese" },
  { value: "other_ethnic_group", label: "Other ethnic group" },
];

const SMOKING_OPTIONS: Array<{ value: Smoking; label: string }> = [
  { value: "non_smoker", label: "Non-smoker" },
  { value: "ex_smoker", label: "Ex-smoker" },
  { value: "light_smoker", label: "Light smoker (<10/day)" },
  { value: "moderate_smoker", label: "Moderate smoker (10-19/day)" },
  { value: "heavy_smoker", label: "Heavy smoker (20+/day)" },
];

const BOOLEAN_FIELDS: Array<{ key: keyof Inputs; label: string }> = [
  { key: "family_history_diabetes", label: "Family history of diabetes" },
  { key: "cardiovascular_disease", label: "Cardiovascular disease" },
  { key: "treated_hypertension", label: "Treated hypertension" },
  { key: "learning_disability", label: "Learning disability" },
  { key: "bipolar_or_schizophrenia", label: "Bipolar disorder or schizophrenia" },
  { key: "regular_steroids", label: "Regular steroid tablets" },
  { key: "statin", label: "On statins" },
  { key: "atypical_antipsychotic", label: "Atypical antipsychotic medication" },
];

function asString(v: unknown): string {
  if (v === null || v === undefined) return "";
  if (typeof v === "string") return v;
  return String(v);
}

function toPayload(inputs: Inputs): Record<string, unknown> {
  const {
    optionalTest,
    fasting_glucose_mmol_l,
    hba1c_mmol_mol,
    polycystic_ovary_syndrome,
    gestational_diabetes,
    ...rest
  } = inputs;

  const payload: Record<string, unknown> = {
    ...rest,
    polycystic_ovary_syndrome: inputs.sex === "female" && polycystic_ovary_syndrome,
    gestational_diabetes: inputs.sex === "female" && gestational_diabetes,
  };
  if (optionalTest === "fasting_glucose") {
    payload.fasting_glucose_mmol_l = fasting_glucose_mmol_l;
  } else if (optionalTest === "hba1c") {
    payload.hba1c_mmol_mol = hba1c_mmol_mol;
  }
  return payload;
}

function buildClipboardSummary(r: CalculationResponse, inputs: Inputs): string {
  const working = r.working ?? {};
  const model = asString(working.model);
  const bmi = asString(working.bmi);

  const riskFactors = BOOLEAN_FIELDS.filter((f) => inputs[f.key])
    .map((f) => `- ${f.label}`)
    .join("\n");
  const femaleFactors = [
    inputs.sex === "female" && inputs.polycystic_ovary_syndrome && "- Polycystic ovary syndrome",
    inputs.sex === "female" && inputs.gestational_diabetes && "- Gestational diabetes",
  ].filter(Boolean) as string[];

  return [
    `QDiabetes-2018 ${asString(r.result)}% (model ${model})`,
    "",
    `Age: ${inputs.age}, sex: ${inputs.sex}, BMI: ${bmi} kg/m2`,
    `Ethnicity: ${inputs.ethnicity}, smoking: ${inputs.smoking}`,
    [...riskFactors.split("\n").filter((l) => l !== ""), ...femaleFactors].join(
      "\n",
    ) || "No additional risk factors.",
    "",
    r.interpretation ?? "",
  ].join("\n");
}

function riskColour(percent: number): string {
  if (percent >= 10) return "red";
  if (percent >= 5) return "yellow";
  return "teal";
}

export function QdiabetesCalculator() {
  const [inputs, setInputs] = useState<Inputs>(blankInputs());
  const [response, setResponse] = useState<CalculationResponse | null>(null);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [clipboardText, setClipboardText] = useState("");

  useEffect(() => {
    let cancelled = false;
    setPending(true);
    setError(null);
    calculate("qdiabetes", toPayload(inputs))
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
  const model = asString(working.model);
  const bmi = asString(working.bmi);
  const riskPercent = Number(response?.result ?? 0);
  const colour = useMemo(() => riskColour(riskPercent), [riskPercent]);

  const reset = () => setInputs(blankInputs());

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(clipboardText);
      notifications.show({
        title: "Copied",
        message: "QDiabetes-2018 result copied.",
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
    <Stack gap="xl" maw={1000}>
      <Group justify="space-between" align="flex-start">
        <Box>
          <Group gap="sm" mb={4}>
            <ThemeIcon size="lg" variant="light" color="teal" radius="md">
              <IconDroplet size={22} />
            </ThemeIcon>
            <Title order={2}>QDiabetes-2018</Title>
            <Badge color="teal" variant="light">
              Type 2 diabetes risk
            </Badge>
          </Group>
          <Text c="dimmed" size="sm">
            10-year risk of developing type 2 diabetes, recommended by NICE
            for identifying patients at high risk from routinely available
            primary-care data.
          </Text>
        </Box>
        <Tooltip label="Reset to defaults">
          <ActionIcon variant="subtle" color="gray" onClick={reset} size="lg">
            <IconRefresh size={18} />
          </ActionIcon>
        </Tooltip>
      </Group>

      <Group align="stretch" gap="lg" wrap="nowrap">
        <Card withBorder padding="lg" radius="lg" style={{ flex: 1.2 }}>
          <Stack gap="md">
            <SimpleGrid cols={3} spacing="md">
              <NumberInput
                label="Age"
                description="25-84"
                min={25}
                max={84}
                value={inputs.age}
                onChange={(value) =>
                  setInputs((prev) => ({ ...prev, age: Number(value) || 0 }))
                }
              />
              <NumberInput
                label="Height"
                description="cm, 140-210"
                min={140}
                max={210}
                value={inputs.height_cm}
                onChange={(value) =>
                  setInputs((prev) => ({
                    ...prev,
                    height_cm: Number(value) || 0,
                  }))
                }
              />
              <NumberInput
                label="Weight"
                description="kg, 40-180"
                min={40}
                max={180}
                value={inputs.weight_kg}
                onChange={(value) =>
                  setInputs((prev) => ({
                    ...prev,
                    weight_kg: Number(value) || 0,
                  }))
                }
              />
            </SimpleGrid>

            <Box>
              <Text fw={500} size="sm" mb={4}>
                Sex
              </Text>
              <SegmentedControl
                fullWidth
                value={inputs.sex}
                onChange={(value) =>
                  setInputs((prev) => ({ ...prev, sex: value as Sex }))
                }
                data={[
                  { value: "male", label: "Male" },
                  { value: "female", label: "Female" },
                ]}
              />
            </Box>

            <SimpleGrid cols={2} spacing="md">
              <Select
                label="Ethnicity"
                data={ETHNICITY_OPTIONS}
                value={inputs.ethnicity}
                onChange={(value) =>
                  value &&
                  setInputs((prev) => ({
                    ...prev,
                    ethnicity: value as Ethnicity,
                  }))
                }
                allowDeselect={false}
              />
              <Select
                label="Smoking"
                data={SMOKING_OPTIONS}
                value={inputs.smoking}
                onChange={(value) =>
                  value &&
                  setInputs((prev) => ({
                    ...prev,
                    smoking: value as Smoking,
                  }))
                }
                allowDeselect={false}
              />
            </SimpleGrid>

            <NumberInput
              label="Townsend deprivation score"
              description="area-based; negative = less deprived, 0 if unknown"
              step={0.1}
              decimalScale={1}
              value={inputs.townsend}
              onChange={(value) =>
                setInputs((prev) => ({
                  ...prev,
                  townsend: Number(value) || 0,
                }))
              }
            />

            <Divider />

            <Text fw={600}>Clinical history</Text>
            <SimpleGrid cols={2} spacing="xs">
              {BOOLEAN_FIELDS.map((f) => (
                <Checkbox
                  key={f.key}
                  size="sm"
                  label={f.label}
                  checked={Boolean(inputs[f.key])}
                  onChange={(e) => {
                    const checked = e.currentTarget.checked;
                    setInputs((prev) => ({ ...prev, [f.key]: checked }));
                  }}
                />
              ))}
            </SimpleGrid>

            {inputs.sex === "female" && (
              <>
                <Divider label="Women only" labelPosition="left" />
                <SimpleGrid cols={2} spacing="xs">
                  <Checkbox
                    size="sm"
                    label="Polycystic ovary syndrome"
                    checked={inputs.polycystic_ovary_syndrome}
                    onChange={(e) => {
                      const checked = e.currentTarget.checked;
                      setInputs((prev) => ({
                        ...prev,
                        polycystic_ovary_syndrome: checked,
                      }));
                    }}
                  />
                  <Checkbox
                    size="sm"
                    label="Gestational diabetes"
                    checked={inputs.gestational_diabetes}
                    onChange={(e) => {
                      const checked = e.currentTarget.checked;
                      setInputs((prev) => ({
                        ...prev,
                        gestational_diabetes: checked,
                      }));
                    }}
                  />
                </SimpleGrid>
              </>
            )}

            <Divider label="Optional: refine with a blood test" labelPosition="left" />
            <SegmentedControl
              fullWidth
              value={inputs.optionalTest}
              onChange={(value) =>
                setInputs((prev) => ({
                  ...prev,
                  optionalTest: value as OptionalTest,
                }))
              }
              data={[
                { value: "none", label: "None (model A)" },
                { value: "fasting_glucose", label: "Fasting glucose (model B)" },
                { value: "hba1c", label: "HbA1c (model C)" },
              ]}
            />
            {inputs.optionalTest === "fasting_glucose" && (
              <NumberInput
                label="Fasting blood glucose"
                description="mmol/L, 2.0-6.99"
                min={2}
                max={6.99}
                step={0.1}
                decimalScale={1}
                value={inputs.fasting_glucose_mmol_l}
                onChange={(value) =>
                  setInputs((prev) => ({
                    ...prev,
                    fasting_glucose_mmol_l: Number(value) || 0,
                  }))
                }
              />
            )}
            {inputs.optionalTest === "hba1c" && (
              <NumberInput
                label="HbA1c"
                description="mmol/mol, 15.0-47.99"
                min={15}
                max={47.99}
                step={0.1}
                decimalScale={1}
                value={inputs.hba1c_mmol_mol}
                onChange={(value) =>
                  setInputs((prev) => ({
                    ...prev,
                    hba1c_mmol_mol: Number(value) || 0,
                  }))
                }
              />
            )}
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
                    %
                  </Text>
                  <Badge ml="auto" color={colour} variant="light" size="lg">
                    model {model}
                  </Badge>
                </Group>
                <Text size="sm" style={{ lineHeight: 1.55 }}>
                  {response.interpretation}
                </Text>
                <Divider my={4} />
                <Badge variant="outline" color="gray" w="fit-content">
                  BMI: {bmi} kg/m²
                </Badge>
                <Alert color="gray" variant="light" title="ClinRisk licence disclaimer">
                  <Text size="xs">{asString(working.disclaimer)}</Text>
                </Alert>
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
