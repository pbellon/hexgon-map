type Scale = (input: number) => number;

type LinearScaleParams = {
  domain: [number, number];
  range: [number, number];
};
export const linearScale = ({ domain, range }: LinearScaleParams): Scale => {
  const [minRange, maxRange] = range;
  const [minDomain, maxDomain] = domain;

  return (input: number) =>
    minRange +
    ((input - minRange) * (maxRange - minRange)) / (maxDomain - minDomain);
};
